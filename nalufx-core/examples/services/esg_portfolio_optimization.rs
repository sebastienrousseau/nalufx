//! # ESG-Screened Portfolio Example
//!
//! Screens a set of tickers by ESG scores you supply, then allocates across
//! the ones that pass with minimum variance (nalufx-engine). NaluFX has no
//! ESG data of its own: the scores are yours, from whichever provider you
//! use, and the screen only applies your threshold to them.
//!
//! Usage: `cargo run --example esg_portfolio_optimization`, then enter
//! the tickers, their scores and a minimum score when prompted.

use nalufx::{
    errors::NaluFxError, services::fetch_data_svc::fetch_returns, utils::input::get_input,
};
use nalufx_engine::allocate::{AllocationInput, Allocator, Constraints, MinVariance};
use nalufx_engine::covariance::CovarianceEstimator;
use nalufx_engine::screening::screen;

#[tokio::main]
pub(crate) async fn main() -> Result<(), NaluFxError> {
    let tickers = tickers("Enter the tickers (comma-separated), e.g. ESGU, ICLN, XOM:")?;
    let scores_input =
        get_input("Enter each ticker's ESG score, in the same order (comma-separated):")?;
    let given: Vec<f64> = scores_input
        .split(',')
        .map(|s| s.trim().parse().map_err(|_| NaluFxError::InvalidOption))
        .collect::<Result<_, _>>()?;
    if given.len() != tickers.len() {
        return Err(NaluFxError::InvalidOption);
    }
    let scored: Vec<(String, f64)> = tickers.iter().cloned().zip(given).collect();
    let min_score: f64 = get_input("Enter the minimum score to keep:")?
        .trim()
        .parse()
        .map_err(|_| NaluFxError::InvalidOption)?;

    let (assets, returns) = fetch_returns(&tickers).await?;
    let scores: Vec<f64> = assets
        .iter()
        .map(|a| scored.iter().find(|(t, _)| t == a).map_or(f64::NAN, |(_, s)| *s))
        .collect();
    let (kept, kept_returns) = screen(&assets, &scores, &returns, min_score)
        .map_err(|e| NaluFxError::PortfolioOptimizationError(e.to_string()))?;
    println!(
        "Kept {} of {} tickers with a score of at least {min_score}: {}",
        kept.len(),
        assets.len(),
        kept.join(", ")
    );

    let input = AllocationInput {
        assets: kept,
        returns: kept_returns,
        constraints: Constraints::default(),
        estimator: CovarianceEstimator::LedoitWolf,
    };
    let allocation = MinVariance
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
