//! # Mean-Variance Portfolio Optimisation Example
//!
//! Fetches daily returns for a set of tickers and allocates across them
//! with the Markowitz mean-variance optimiser from nalufx-engine:
//! maximise `mu' w - (lambda / 2) w' S w`, long-only, budget 1, within a
//! per-asset cap, with a Ledoit-Wolf covariance.
//!
//! Usage: `cargo run --example mean_variance_optimization`, then enter the
//! tickers, a risk aversion and a cap when prompted.

use nalufx::{
    errors::NaluFxError, services::fetch_data_svc::fetch_returns, utils::input::get_input,
};
use nalufx_engine::allocate::{AllocationInput, Allocator, Constraints, MeanVariance};
use nalufx_engine::covariance::CovarianceEstimator;

#[tokio::main]
pub(crate) async fn main() -> Result<(), NaluFxError> {
    let tickers = tickers("Enter the tickers (comma-separated), e.g. AAPL, MSFT, GOOGL, TLT:")?;
    let risk_aversion: f64 =
        get_input("Enter the risk aversion (e.g. 10):")?.trim().parse().unwrap_or(10.0);
    let max_weight: f64 =
        get_input("Enter the largest weight per asset (e.g. 0.4):")?.trim().parse().unwrap_or(1.0);
    let (assets, returns) = fetch_returns(&tickers).await?;
    let method =
        MeanVariance::new(risk_aversion).map_err(|e| NaluFxError::NaluFxError(e.to_string()))?;
    let input = AllocationInput {
        assets,
        returns,
        constraints: Constraints { max_weight, min_weight: 0.0 },
        estimator: CovarianceEstimator::LedoitWolf,
    };
    let allocation = method
        .allocate(&input)
        .map_err(|e| NaluFxError::PortfolioOptimizationError(e.to_string()))?;
    println!("\n{allocation}\n\nThis is research output, not financial advice.");
    Ok(())
}

/// Prompts for comma-separated tickers.
fn tickers(prompt: &str) -> Result<Vec<String>, NaluFxError> {
    let input = get_input(prompt)?;
    Ok(input.split(',').map(|s| s.trim().to_uppercase()).filter(|s| !s.is_empty()).collect())
}
