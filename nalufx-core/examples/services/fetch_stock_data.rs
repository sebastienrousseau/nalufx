//! # Stock Data Fetching and Allocation Example
//!
//! Fetches the closing prices of a stock, computes its daily returns, and
//! allocates an investment across the next few trading days by inverse
//! forecast volatility.
//!
//! Usage:
//! 1. Run `cargo run --example fetch_stock_data`.
//! 2. Enter a ticker symbol and an investment amount when prompted.
use nalufx::{
    errors::NaluFxError,
    services::{fetch_data_svc::fetch_data, processing_svc::calculate_daily_returns},
    utils::{
        calculations::{calculate_optimal_allocation, DEFAULT_HORIZON_DAYS},
        currency::format_currency,
        date::format_date,
        input::get_input,
        ticker::validate_ticker,
        validation::validate_positive_float,
    },
};
use time::{Duration, OffsetDateTime};

#[tokio::main]
pub(crate) async fn main() -> Result<(), NaluFxError> {
    let ticker_input = get_input("Enter the ticker symbol:")?;
    let ticker = validate_ticker(&ticker_input).map_err(|e| {
        eprintln!("Error: {e}");
        NaluFxError::InvalidOption
    })?;
    let investment_input = get_input("Enter the initial investment amount:")?;
    let investment = validate_positive_float(&investment_input).map_err(|e| {
        eprintln!("Error: {e}");
        NaluFxError::InvalidOption
    })?;

    let closes = fetch_data(ticker, None, None).await.map_err(|e| {
        eprintln!("Error fetching data for ticker {ticker}: {e}");
        NaluFxError::InvalidOption
    })?;
    let daily_returns = calculate_daily_returns(&closes)
        .map_err(|e| NaluFxError::PortfolioOptimizationError(e.to_string()))?;
    let allocation = calculate_optimal_allocation(&daily_returns, DEFAULT_HORIZON_DAYS)
        .map_err(|e| NaluFxError::PortfolioOptimizationError(e.to_string()))?;

    println!("\n--- Allocation for {ticker} ---\n");
    println!(
        "{} daily returns, forecast {DEFAULT_HORIZON_DAYS} days ahead with AutoETS; each day is \
         weighted by the inverse of its 95% forecast interval's width.\n",
        daily_returns.len()
    );
    let today = OffsetDateTime::now_utc();
    for (i, weight) in allocation.iter().enumerate() {
        let date = format_date(today + Duration::days(i64::try_from(i).unwrap_or(i64::MAX)));
        let amount = format_currency(weight * investment);
        println!("- Day {}: {date} - Allocate {amount} ({:.2}%)", i + 1, weight * 100.0);
    }
    println!("\nThis is research output, not financial advice.");
    Ok(())
}
