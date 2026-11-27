//! # Risk Parity Portfolio Optimisation Example
//!
//! Fetches daily returns for a set of tickers and allocates across them so
//! each contributes the same share of the portfolio's variance (equal
//! risk contribution, from nalufx-engine), and compares it with
//! hierarchical risk parity on the same data.
//!
//! Usage: `cargo run --example risk_parity_portfolio_optimization`, then
//! enter the tickers when prompted.

use nalufx::{
    errors::NaluFxError, services::fetch_data_svc::fetch_returns, utils::input::get_input,
};
use nalufx_engine::allocate::{AllocationInput, Allocator, Constraints, Hrp, RiskParity};
use nalufx_engine::covariance::CovarianceEstimator;

#[tokio::main]
pub(crate) async fn main() -> Result<(), NaluFxError> {
    let tickers = tickers("Enter the tickers (comma-separated), e.g. SPY, TLT, GLD, VNQ:")?;
    let (assets, returns) = fetch_returns(&tickers).await?;
    let input = AllocationInput {
        assets,
        returns,
        constraints: Constraints::default(),
        estimator: CovarianceEstimator::LedoitWolf,
    };
    let methods: [&dyn Allocator; 2] = [&RiskParity, &Hrp];
    for method in methods {
        let allocation = method
            .allocate(&input)
            .map_err(|e| NaluFxError::PortfolioOptimizationError(e.to_string()))?;
        println!("\n{allocation}\n\nThis is research output, not financial advice.");
    }
    Ok(())
}

/// Prompts for comma-separated tickers.
fn tickers(prompt: &str) -> Result<Vec<String>, NaluFxError> {
    let input = get_input(prompt)?;
    Ok(input.split(',').map(|s| s.trim().to_uppercase()).filter(|s| !s.is_empty()).collect())
}
