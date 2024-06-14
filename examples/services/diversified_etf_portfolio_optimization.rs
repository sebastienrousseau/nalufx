//! # Diversified ETF Portfolio Optimization Example
//!
//! This example demonstrates how to optimize a diversified portfolio of ETFs (Exchange-Traded Funds).
//! It fetches historical closing prices for user-specified ETFs, calculates daily returns, cash flows,
//! and performs sentiment analysis, reinforcement learning, and optimal allocation for each ETF.
//! The code compares the performance of the ETFs and selects the best-performing one based on a custom metric.
//! The results are presented in a comprehensive report format.
//!
//! Usage:
//! 1. Run the code using `cargo run --example diversified_etf_portfolio_optimization`.
//! 2. Enter the ticker symbols for ETFs separated by commas (e.g., SPY,GLD) when prompted.
//! 3. Enter the initial investment amount when prompted.
//! 4. The code will fetch historical data for each ETF, perform analysis, and generate a report with investment recommendations for the best-performing ETF.
use nalufx::{
    errors::NaluFxError,
    utils::{
        input::get_input, ticker::validate_ticker, validation::validate_positive_float
    },
};
use nalufx::services::diversified_etf_portfolio_optimization_svc::generate_analysis;

#[tokio::main]
pub(crate) async fn main() -> Result<(), NaluFxError> {
    let tickers_input = get_input("Enter the ticker symbols separated by commas (e.g., SQQQ,SPY,SOXL,XLF):")?;
    let tickers: Vec<String> = tickers_input
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    for ticker in &tickers {
        if let Err(e) = validate_ticker(ticker) {
            eprintln!("Error: {}", e);
            return Ok(());
        }
    }

    let initial_investment_input = get_input("Enter the initial investment amount:")?;
    let initial_investment = match validate_positive_float(&initial_investment_input) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(NaluFxError::InvalidOption);
        }
    };

    generate_analysis(tickers, initial_investment).await
}
