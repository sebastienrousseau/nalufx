//! Allocation across assets.
//!
//! Each method is a type implementing [`Allocator`];
//! [`by_name`] picks one
//! from a string, as the API does. Every
//! [`Allocation`] records the method,
//! its parameters and the solver's diagnostics, so a result can always be
//! traced back to what produced it.
//!
//! Every method is long-only with a budget of 1 and honours a per-asset
//! cap and floor ([`Constraints`]). Minimum
//! variance and mean-variance
//! solve them exactly as a quadratic programme. Risk parity and HRP have
//! no native form for them, so their weights are projected onto the
//! bounds afterwards ([`project`]), which
//! [`Diagnostics::projected`]
//! reports.

mod equal;
mod hrp;
mod qp;
mod risk_parity;

pub use equal::EqualWeight;
pub use hrp::Hrp;
pub use qp::{MeanVariance, MinVariance};
pub use risk_parity::RiskParity;

use crate::covariance::{CovarianceEstimator, Moments};
use crate::linalg::condition_number;
use crate::AllocationError;
use ndarray::Array2;
use serde::{Deserialize, Serialize};

/// Bounds on each weight. Every allocation is also long-only and sums to 1.
/// A bound left out when deserialising takes its default (no cap, no floor).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Constraints {
    /// The largest weight any one asset may have, in `(0, 1]`.
    pub max_weight: f64,
    /// The smallest weight every asset must have, in `[0, max_weight]`.
    pub min_weight: f64,
}

impl Default for Constraints {
    fn default() -> Self {
        Self { max_weight: 1.0, min_weight: 0.0 }
    }
}

impl Constraints {
    /// Checks that some weights over `n` assets satisfy the bounds.
    ///
    /// # Errors
    ///
    /// - [`AllocationError::InvalidParameter`] if a bound is not finite, or
    ///   outside `0 <= min_weight <= max_weight <= 1`.
    /// - [`AllocationError::Infeasible`] if `n` caps cannot reach 1, or `n`
    ///   floors exceed it.
    ///
    /// # Examples
    ///
    /// ```
    /// use nalufx_engine::allocate::Constraints;
    ///
    /// let c = Constraints { max_weight: 0.3, min_weight: 0.0 };
    /// assert!(c.check(4).is_ok());
    /// assert!(c.check(3).is_err()); // three 30% caps reach only 90%
    /// ```
    pub fn check(&self, n: usize) -> Result<(), AllocationError> {
        let (lo, hi) = (self.min_weight, self.max_weight);
        if !(lo.is_finite() && hi.is_finite() && 0.0 <= lo && lo <= hi && hi <= 1.0 && hi > 0.0) {
            return Err(AllocationError::InvalidParameter(format!(
                "weight bounds must satisfy 0 <= min ({lo}) <= max ({hi}) <= 1 with max > 0"
            )));
        }
        let nf = n as f64;
        if nf * hi < 1.0 - BUDGET_TOLERANCE {
            return Err(AllocationError::Infeasible(format!(
                "{n} assets capped at {hi} cannot reach a budget of 1"
            )));
        }
        if nf * lo > 1.0 + BUDGET_TOLERANCE {
            return Err(AllocationError::Infeasible(format!(
                "{n} assets with a floor of {lo} exceed a budget of 1"
            )));
        }
        Ok(())
    }
}

/// How far a sum may stray from the budget of 1 and still count as 1.
const BUDGET_TOLERANCE: f64 = 1e-12;

/// What an allocation is computed from.
#[derive(Clone, Debug, PartialEq)]
pub struct AllocationInput {
    /// The asset names, one per column of `returns`.
    pub assets: Vec<String>,
    /// One row per period, one column per asset.
    pub returns: Array2<f64>,
    /// The weight bounds.
    pub constraints: Constraints,
    /// How the covariance is estimated.
    pub estimator: CovarianceEstimator,
}

/// How the weights were found.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Diagnostics {
    /// The solver's final status, such as `"solved"` or `"closed_form"`.
    pub solver_status: String,
    /// Solver iterations; 0 for a closed form.
    pub iterations: u32,
    /// The 2-norm condition number of the covariance used.
    pub condition_number: f64,
    /// The Ledoit-Wolf shrinkage intensity, if that estimator was used.
    pub shrinkage: Option<f64>,
    /// Whether the method's weights were projected onto the bounds
    /// afterwards, which moves them off the method's own optimum.
    pub projected: bool,
}

/// An allocation and everything needed to reproduce it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Allocation {
    /// The asset names, in the order of `weights`.
    pub assets: Vec<String>,
    /// One weight per asset: each within the bounds, summing to 1.
    pub weights: Vec<f64>,
    /// The method's name, as [`by_name`] takes it.
    pub method: String,
    /// The method's parameters, the bounds and the estimator.
    pub params: serde_json::Value,
    /// How the weights were found.
    pub diagnostics: Diagnostics,
}

impl std::fmt::Display for Allocation {
    /// A plain-text summary: one line per asset, then the diagnostics.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} allocation", self.method)?;
        for (asset, w) in self.assets.iter().zip(&self.weights) {
            writeln!(f, "  {asset}: {:.2}%", w * 100.0)?;
        }
        let d = &self.diagnostics;
        let shrinkage = d.shrinkage.map_or_else(|| "none".to_string(), |s| format!("{s:.3}"));
        write!(
            f,
            "solver {} after {} iterations; condition number {:.1}; shrinkage {shrinkage}{}",
            d.solver_status,
            d.iterations,
            d.condition_number,
            if d.projected { "; projected onto the bounds" } else { "" }
        )
    }
}

/// The weights a method computes from moments, and how it found them.
#[derive(Clone, Debug, PartialEq)]
pub struct Solution {
    /// One weight per asset, within the bounds and summing to 1.
    pub weights: Vec<f64>,
    /// The solver's final status.
    pub solver_status: String,
    /// Solver iterations; 0 for a closed form.
    pub iterations: u32,
    /// Whether the weights were projected onto the bounds afterwards.
    pub projected: bool,
}

/// An allocation method.
pub trait Allocator {
    /// The method's name, as [`by_name`] takes it.
    fn name(&self) -> &'static str;

    /// The method's own parameters, for the audit trail.
    fn params(&self) -> serde_json::Value;

    /// The weights for given moments and bounds. The bounds have already
    /// passed [`Constraints::check`].
    ///
    /// # Errors
    ///
    /// Whatever the method's solver reports.
    fn solve(
        &self,
        moments: &Moments,
        constraints: &Constraints,
    ) -> Result<Solution, AllocationError>;

    /// Estimates the moments of the input and allocates.
    ///
    /// # Errors
    ///
    /// - [`AllocationError::InputMismatch`] if the names do not match the columns.
    /// - Any error from [`CovarianceEstimator::estimate`], [`Constraints::check`]
    ///   or the method's solver.
    fn allocate(&self, input: &AllocationInput) -> Result<Allocation, AllocationError> {
        if input.assets.len() != input.returns.ncols() {
            return Err(AllocationError::InputMismatch);
        }
        let moments = input.estimator.estimate(&input.returns)?;
        input.constraints.check(input.assets.len())?;
        let solution = self.solve(&moments, &input.constraints)?;
        let diagnostics = Diagnostics {
            solver_status: solution.solver_status,
            iterations: solution.iterations,
            condition_number: condition_number(&moments.covariance)?,
            shrinkage: moments.shrinkage,
            projected: solution.projected,
        };
        Ok(Allocation {
            assets: input.assets.clone(),
            weights: solution.weights,
            method: self.name().to_string(),
            params: serde_json::json!({
                "method": self.params(),
                "constraints": input.constraints,
                "estimator": input.estimator,
            }),
            diagnostics,
        })
    }
}

/// The method names [`by_name`] accepts.
pub const METHODS: [&str; 5] =
    ["equal_weight", "min_variance", "mean_variance", "risk_parity", "hrp"];

/// The allocator with this name, with default parameters except
/// `risk_aversion`, which only `mean_variance` takes.
///
/// # Errors
///
/// - [`AllocationError::UnknownMethod`] for a name not in [`METHODS`].
/// - [`AllocationError::InvalidParameter`] for a risk aversion that is not
///   finite and positive.
///
/// # Examples
///
/// ```
/// use nalufx_engine::allocate::by_name;
///
/// assert_eq!(by_name("hrp", None).unwrap().name(), "hrp");
/// assert!(by_name("mean_variance", Some(-1.0)).is_err());
/// assert!(by_name("magic", None).is_err());
/// ```
pub fn by_name(
    name: &str,
    risk_aversion: Option<f64>,
) -> Result<Box<dyn Allocator>, AllocationError> {
    Ok(match name {
        "equal_weight" => Box::new(EqualWeight),
        "min_variance" => Box::new(MinVariance),
        "mean_variance" => Box::new(MeanVariance::new(
            risk_aversion.unwrap_or(MeanVariance::DEFAULT_RISK_AVERSION),
        )?),
        "risk_parity" => Box::new(RiskParity),
        "hrp" => Box::new(Hrp),
        other => return Err(AllocationError::UnknownMethod(other.to_string())),
    })
}

/// Projects weights onto the bounds while keeping a budget of 1.
///
/// The result is the nearest point, in Euclidean distance, whose weights
/// are within the bounds and sum to 1: `clamp(w_i - tau, min, max)` for
/// the one shift `tau` that makes them sum to 1, found by bisection.
/// Weights already within the bounds and summing to 1 come back unchanged.
///
/// # Errors
///
/// As [`Constraints::check`], and [`AllocationError::InvalidData`] for a
/// weight that is NaN or infinite.
///
/// # Examples
///
/// ```
/// use nalufx_engine::allocate::{project, Constraints};
///
/// let c = Constraints { max_weight: 0.5, min_weight: 0.0 };
/// let w = project(&[0.8, 0.15, 0.05], &c).unwrap();
/// assert_eq!(w[0], 0.5);
/// assert!((w[1] - 0.3).abs() < 1e-12 && (w[2] - 0.2).abs() < 1e-12);
/// ```
pub fn project(weights: &[f64], constraints: &Constraints) -> Result<Vec<f64>, AllocationError> {
    constraints.check(weights.len())?;
    if weights.iter().any(|x| !x.is_finite()) {
        return Err(AllocationError::InvalidData);
    }
    let bounds = (constraints.min_weight, constraints.max_weight);
    let tau = if total(weights, bounds, 0.0) == 1.0 { 0.0 } else { budget_shift(weights, bounds) };
    Ok(weights.iter().map(|w| (w - tau).clamp(bounds.0, bounds.1)).collect())
}

/// The sum of `clamp(w_i - tau, min, max)`; it falls as `tau` rises.
fn total(weights: &[f64], (lo, hi): (f64, f64), tau: f64) -> f64 {
    weights.iter().map(|w| (w - tau).clamp(lo, hi)).sum()
}

/// The shift `tau` at which [`total`] is 1, by bisection between the
/// shift that puts every weight at the cap (`n * max >= 1`) and the one
/// that puts every weight at the floor (`n * min <= 1`).
fn budget_shift(weights: &[f64], (lo, hi): (f64, f64)) -> f64 {
    /// Bisection steps: each halves the bracket, so 200 reach the limit
    /// of f64 resolution from any bracket a weight vector can produce.
    const STEPS: usize = 200;
    let smallest = weights.iter().copied().fold(f64::INFINITY, f64::min);
    let largest = weights.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let (mut below, mut above) = (smallest - hi, largest - lo);
    for _ in 0..STEPS {
        let mid = 0.5 * (below + above);
        if total(weights, (lo, hi), mid) > 1.0 {
            below = mid;
        } else {
            above = mid;
        }
    }
    0.5 * (below + above)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constraints_reject_bad_bounds() {
        for (hi, lo) in [(0.0, 0.0), (1.1, 0.0), (0.5, 0.6), (f64::NAN, 0.0), (0.5, -0.1)] {
            let c = Constraints { max_weight: hi, min_weight: lo };
            assert!(matches!(c.check(4), Err(AllocationError::InvalidParameter(_))), "{hi} {lo}");
        }
    }

    #[test]
    fn constraints_reject_an_unreachable_budget() {
        assert!(matches!(
            Constraints { max_weight: 0.2, min_weight: 0.0 }.check(4),
            Err(AllocationError::Infeasible(_))
        ));
        assert!(matches!(
            Constraints { max_weight: 1.0, min_weight: 0.3 }.check(4),
            Err(AllocationError::Infeasible(_))
        ));
        assert!(Constraints { max_weight: 0.25, min_weight: 0.25 }.check(4).is_ok());
    }

    #[test]
    fn projection_leaves_feasible_weights_alone() {
        let w = [0.2, 0.3, 0.5];
        assert_eq!(project(&w, &Constraints::default()).unwrap(), w.to_vec());
    }

    #[test]
    fn projection_reaches_the_budget_when_every_weight_is_clamped() {
        // Floors and caps both bind: [1, 0, 0] with bounds [0.2, 0.5].
        let c = Constraints { max_weight: 0.5, min_weight: 0.2 };
        let w = project(&[1.0, 0.0, 0.0], &c).unwrap();
        assert!((w.iter().sum::<f64>() - 1.0).abs() < 1e-12, "{w:?}");
        assert!(w.iter().all(|x| (0.2..=0.5).contains(x)), "{w:?}");
    }

    #[test]
    fn projection_enforces_a_floor() {
        let c = Constraints { max_weight: 1.0, min_weight: 0.1 };
        let w = project(&[0.9, 0.1, 0.0], &c).unwrap();
        assert!((w[2] - 0.1).abs() < 1e-15);
        assert!((w.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        assert!(w.iter().all(|x| *x >= 0.1 - 1e-15 && *x <= 1.0));
    }

    #[test]
    fn projection_cascades_when_redistribution_breaks_a_cap() {
        let c = Constraints { max_weight: 0.4, min_weight: 0.0 };
        let w = project(&[0.7, 0.25, 0.05], &c).unwrap();
        // The shift that caps 0.7 at 0.4 pushes 0.25 over the cap too.
        assert!((w[0] - 0.4).abs() < 1e-15 && (w[1] - 0.4).abs() < 1e-15);
        assert!((w[2] - 0.2).abs() < 1e-12);
    }

    #[test]
    fn every_method_name_resolves() {
        for name in METHODS {
            assert_eq!(by_name(name, None).unwrap().name(), name);
        }
    }
}
