//! Walk-forward backtests of an allocation method.
//!
//! At each rebalance the method sees only the trailing `lookback`
//! periods, so no result uses data from its own future. Between
//! rebalances the weights drift with the assets' returns, and each
//! rebalance pays its turnover times the cost.

use crate::allocate::{AllocationInput, Allocator, Constraints};
use crate::covariance::CovarianceEstimator;
use crate::AllocationError;
use ndarray::{s, Array2, ArrayView1};
use serde::{Deserialize, Serialize};

/// How a backtest runs.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BacktestConfig {
    /// Periods of history each allocation is estimated from.
    pub lookback: usize,
    /// Periods between rebalances.
    pub rebalance_every: usize,
    /// The cost of trading, in basis points of the value traded.
    pub cost_bps: f64,
    /// Periods per year, for annualising (252 for trading days).
    pub periods_per_year: f64,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self { lookback: 120, rebalance_every: 21, cost_bps: 10.0, periods_per_year: 252.0 }
    }
}

/// What a backtest measured, net of costs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BacktestReport {
    /// The method's name.
    pub method: String,
    /// The method's parameters.
    pub params: serde_json::Value,
    /// The configuration the backtest ran with.
    pub config: BacktestConfig,
    /// Periods with a portfolio return (after the first lookback).
    pub periods: usize,
    /// Rebalances made, the first one included.
    pub rebalances: usize,
    /// The compound annual growth rate.
    pub annualised_return: f64,
    /// The standard deviation of period returns, annualised.
    pub annualised_volatility: f64,
    /// The annualised mean return over the annualised volatility, with a
    /// zero risk-free rate; 0 if the volatility is 0.
    pub sharpe: f64,
    /// The largest fall from a peak of the value, as a positive fraction.
    pub max_drawdown: f64,
    /// The value traded per year (bought plus sold), as a fraction of the
    /// portfolio: the sum of absolute weight changes. The first rebalance
    /// buys in from cash, which counts as 1.
    pub annual_turnover: f64,
}

/// Runs a walk-forward backtest.
///
/// # Arguments
///
/// * `allocator` - The method to test.
/// * `assets` - The asset names, one per column of `returns`.
/// * `returns` - One row per period, one column per asset.
/// * `constraints` - The bounds every allocation must meet.
/// * `estimator` - How each allocation's covariance is estimated.
/// * `config` - The lookback, rebalance schedule, cost and annualisation.
///
/// # Errors
///
/// - [`AllocationError::InvalidParameter`] if the lookback is under 2, the
///   rebalance interval is 0, the cost is negative or not finite, the year
///   is not positive, or there is no period after the first lookback.
/// - Any error from the method at a rebalance.
pub fn run(
    allocator: &dyn Allocator,
    assets: &[String],
    returns: &Array2<f64>,
    constraints: Constraints,
    estimator: CovarianceEstimator,
    config: BacktestConfig,
) -> Result<BacktestReport, AllocationError> {
    check_config(&config, returns.nrows())?;
    let cost = config.cost_bps / 10_000.0;
    let mut walk = Walk::new(returns.ncols(), returns.nrows() - config.lookback);
    for t in config.lookback..returns.nrows() {
        let charge = if (t - config.lookback).is_multiple_of(config.rebalance_every) {
            let input = AllocationInput {
                assets: assets.to_vec(),
                returns: returns.slice(s![t - config.lookback..t, ..]).to_owned(),
                constraints,
                estimator,
            };
            walk.rebalance(allocator.allocate(&input)?.weights) * cost
        } else {
            0.0
        };
        walk.step(returns.row(t), charge);
    }
    Ok(summarise(allocator, config, &walk))
}

/// The state of a backtest as it walks forward.
struct Walk {
    /// The current weights, drifted since the last rebalance.
    weights: Vec<f64>,
    /// Rebalances so far.
    rebalances: usize,
    /// Value traded so far, as a fraction of the portfolio.
    traded: f64,
    /// The net return of each period so far.
    period_returns: Vec<f64>,
}

impl Walk {
    /// A walk starting in cash.
    fn new(assets: usize, periods: usize) -> Self {
        Self {
            weights: vec![0.0; assets],
            rebalances: 0,
            traded: 0.0,
            period_returns: Vec::with_capacity(periods),
        }
    }

    /// Moves to the target weights; returns the value traded (bought plus
    /// sold), as a fraction of the portfolio.
    fn rebalance(&mut self, target: Vec<f64>) -> f64 {
        let turnover: f64 = target.iter().zip(&self.weights).map(|(a, b)| (a - b).abs()).sum();
        self.traded += turnover;
        self.rebalances += 1;
        self.weights = target;
        turnover
    }

    /// Records one period's net return and lets the weights drift with it.
    fn step(&mut self, returns: ArrayView1<'_, f64>, charge: f64) {
        let gross: f64 = self.weights.iter().zip(returns).map(|(w, r)| w * r).sum();
        self.period_returns.push(gross - charge);
        let growth = 1.0 + gross;
        if growth > 0.0 {
            for (w, r) in self.weights.iter_mut().zip(returns) {
                *w *= (1.0 + r) / growth;
            }
        }
    }
}

/// Rejects a configuration the backtest cannot run.
fn check_config(config: &BacktestConfig, periods: usize) -> Result<(), AllocationError> {
    let bad = |what: &str| Err(AllocationError::InvalidParameter(what.to_string()));
    if config.lookback < 2 {
        return bad("the lookback must be at least 2 periods");
    }
    if config.rebalance_every == 0 {
        return bad("the rebalance interval must be at least 1 period");
    }
    if !(config.cost_bps.is_finite() && config.cost_bps >= 0.0) {
        return bad("the cost must be finite and not negative");
    }
    if !(config.periods_per_year.is_finite() && config.periods_per_year > 0.0) {
        return bad("periods per year must be finite and positive");
    }
    if periods <= config.lookback {
        return bad("there must be at least one period after the lookback");
    }
    Ok(())
}

/// The report's statistics from the walk.
fn summarise(allocator: &dyn Allocator, config: BacktestConfig, walk: &Walk) -> BacktestReport {
    let r = &walk.period_returns;
    let years = r.len() as f64 / config.periods_per_year;
    let (value, max_drawdown) = growth_and_drawdown(r);
    let (mean, annualised_volatility) = mean_and_volatility(r, config.periods_per_year);
    BacktestReport {
        method: allocator.name().to_string(),
        params: allocator.params(),
        config,
        periods: r.len(),
        rebalances: walk.rebalances,
        annualised_return: if value > 0.0 { value.powf(1.0 / years) - 1.0 } else { -1.0 },
        annualised_volatility,
        sharpe: if annualised_volatility > 0.0 {
            mean * config.periods_per_year / annualised_volatility
        } else {
            0.0
        },
        max_drawdown,
        annual_turnover: walk.traded / years,
    }
}

/// The final value of 1 invested, and the largest fall from a peak.
fn growth_and_drawdown(r: &[f64]) -> (f64, f64) {
    let (mut value, mut peak, mut max_drawdown) = (1.0f64, 1.0f64, 0.0f64);
    for x in r {
        value *= 1.0 + x;
        peak = peak.max(value);
        max_drawdown = max_drawdown.max(1.0 - value / peak);
    }
    (value, max_drawdown)
}

/// The mean period return and the annualised sample volatility.
fn mean_and_volatility(r: &[f64], periods_per_year: f64) -> (f64, f64) {
    let n = r.len() as f64;
    let mean = r.iter().sum::<f64>() / n;
    if r.len() < 2 {
        return (mean, 0.0);
    }
    let variance = r.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    (mean, (variance * periods_per_year).sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocate::EqualWeight;
    use ndarray::Array2;

    fn names(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("A{i}")).collect()
    }

    /// Two assets that each return 1% a period: the portfolio does too,
    /// less the one initial trade.
    #[test]
    fn a_constant_return_compounds() {
        let returns = Array2::from_elem((30, 2), 0.01);
        let config = BacktestConfig {
            lookback: 10,
            rebalance_every: 5,
            cost_bps: 0.0,
            periods_per_year: 252.0,
        };
        let report = run(
            &EqualWeight,
            &names(2),
            &returns,
            Constraints::default(),
            CovarianceEstimator::Sample,
            config,
        );
        // The sample covariance of a constant series is 0, which equal
        // weight never looks at.
        let report = report.unwrap();
        assert_eq!(report.periods, 20);
        assert_eq!(report.rebalances, 4);
        assert!((report.annualised_return - (1.01f64.powf(252.0) - 1.0)).abs() < 1e-9);
        assert_eq!(report.max_drawdown, 0.0);
        // The first rebalance buys everything (traded 1); weights never
        // drift apart, so later ones trade nothing.
        assert!((report.annual_turnover - 1.0 / (20.0 / 252.0)).abs() < 1e-9);
    }

    #[test]
    fn costs_are_charged_on_turnover() {
        let returns = Array2::from_elem((12, 2), 0.0);
        let config = BacktestConfig {
            lookback: 2,
            rebalance_every: 100,
            cost_bps: 100.0,
            periods_per_year: 252.0,
        };
        let report = run(
            &EqualWeight,
            &names(2),
            &returns,
            Constraints::default(),
            CovarianceEstimator::Sample,
            config,
        )
        .unwrap();
        // One trade of the whole portfolio at 1%: value 0.99.
        assert!((report.annualised_return - (0.99f64.powf(252.0 / 10.0) - 1.0)).abs() < 1e-12);
        assert!((report.max_drawdown - 0.01).abs() < 1e-12);
    }

    #[test]
    fn a_bad_configuration_is_rejected() {
        let returns = Array2::from_elem((12, 2), 0.0);
        for config in [
            BacktestConfig { lookback: 1, ..BacktestConfig::default() },
            BacktestConfig { lookback: 5, rebalance_every: 0, ..BacktestConfig::default() },
            BacktestConfig { lookback: 5, cost_bps: -1.0, ..BacktestConfig::default() },
            BacktestConfig { lookback: 20, ..BacktestConfig::default() },
        ] {
            let result = run(
                &EqualWeight,
                &names(2),
                &returns,
                Constraints::default(),
                CovarianceEstimator::Sample,
                config,
            );
            assert!(matches!(result, Err(AllocationError::InvalidParameter(_))), "{config:?}");
        }
    }
}
