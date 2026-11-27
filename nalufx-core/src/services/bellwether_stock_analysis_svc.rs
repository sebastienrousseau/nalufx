//! Bellwether stock report: an inverse-volatility allocation over the
//! next few days, with a language model's commentary on it.
//!
//! The numbers come from the maths in
//! [`calculate_optimal_allocation`](crate::utils::calculations::calculate_optimal_allocation);
//! the language model writes words about them and nothing else.

use crate::{
    errors::NaluFxError,
    services::{fetch_data_svc::fetch_data, processing_svc::calculate_daily_returns},
    utils::{
        calculations::{calculate_optimal_allocation, DEFAULT_HORIZON_DAYS},
        date::validate_date,
    },
};
use nalufx_llms::llms::LLM;
use reqwest::Client;
use time::OffsetDateTime;

/// Generates a report on one stock from its price history.
///
/// # Arguments
///
/// * `llm` - The language model that writes the commentary.
/// * `client` - The HTTP client used to reach the model provider.
/// * `api_key` - The API key for the model provider.
/// * `ticker` - The ticker symbol of the stock to analyse.
/// * `initial_investment` - The amount the allocation is applied to.
/// * `start_date` - The start of the price history, "YYYY-MM-DD".
/// * `end_date` - The end of the price history, "YYYY-MM-DD".
///
/// # Errors
///
/// - [`NaluFxError::InvalidOption`] if a date is invalid or no history is available.
/// - [`NaluFxError::PortfolioOptimizationError`] if the prices or the allocation fail.
/// - Any error from the model provider.
pub async fn generate_analysis(
    llm: Box<dyn LLM>,
    client: &Client,
    api_key: &str,
    ticker: &str,
    initial_investment: f64,
    start_date: &str,
    end_date: &str,
) -> Result<(), NaluFxError> {
    let start = validate_date(start_date).map_err(invalid_option)?;
    let end = validate_date(end_date).map_err(invalid_option)?;
    let closes = fetch_data(ticker, Some(start), Some(end)).await.map_err(|e| {
        eprintln!("Historical data not available for {ticker} in that date range: {e}");
        println!("Please try a different date range or choose another ticker symbol.");
        NaluFxError::InvalidOption
    })?;
    if closes.is_empty() {
        eprintln!("No closing prices found for ticker {ticker} in the specified date range");
        return Ok(());
    }

    let allocation = allocate(&closes)?;
    let prompt = format!(
        "Analyze the following stock data for {ticker}:\n\n\
         - Allocation over the next {DEFAULT_HORIZON_DAYS} trading days (fractions of the \
         investment, by inverse forecast volatility): {allocation:?}\n\n\
         Provide an investment commentary on this data, and the current market context for \
         {ticker} in {year}.",
        year = OffsetDateTime::now_utc().year(),
    );
    let response = llm.send_request(client, api_key, &prompt, 1500).await?;
    let commentary = response["choices"][0]["message"]["content"].as_str().unwrap_or("");

    let dates = (start_date, end_date);
    print_report(&Report {
        ticker,
        dates,
        initial_investment,
        allocation: &allocation,
        commentary,
    });
    Ok(())
}

/// The daily inverse-volatility allocation for a price history.
fn allocate(closes: &[f64]) -> Result<Vec<f64>, NaluFxError> {
    calculate_daily_returns(closes)
        .and_then(|returns| calculate_optimal_allocation(&returns, DEFAULT_HORIZON_DAYS))
        .map_err(|e| NaluFxError::PortfolioOptimizationError(e.to_string()))
}

/// Logs an invalid date and turns it into [`NaluFxError::InvalidOption`].
fn invalid_option(err: impl std::fmt::Display) -> NaluFxError {
    eprintln!("Error: {err}");
    NaluFxError::InvalidOption
}

/// What the report prints.
#[derive(Debug, Clone, Copy)]
struct Report<'a> {
    ticker: &'a str,
    dates: (&'a str, &'a str),
    initial_investment: f64,
    allocation: &'a [f64],
    commentary: &'a str,
}

/// Prints the report.
fn print_report(r: &Report<'_>) {
    let ticker = r.ticker;
    println!("\n--- Bellwether Stock Report: {ticker} ---\n");
    println!("- **Date Range:** {} - {}", r.dates.0, r.dates.1);
    println!("- **Initial Investment:** ${:.2}\n", r.initial_investment);

    println!("--- Methodology ---\n");
    println!("The daily returns of {ticker} over the date range are forecast {DEFAULT_HORIZON_DAYS} trading days ahead with an exponential smoothing model (AutoETS). Each of those days is weighted by the inverse of its 95% forecast interval's width, so the days the model is surer about get more of the investment. The weights sum to 100%. The method uses the uncertainty of the forecast only, not its expected return.\n");

    println!("--- Allocation ---\n");
    for (day, weight) in r.allocation.iter().enumerate() {
        let amount = weight * r.initial_investment;
        println!("- Day {}: {:.2}% (${amount:.2})", day + 1, weight * 100.0);
    }
    let (low, high) = range(r.allocation);
    println!("\nDaily weights range from {:.2}% to {:.2}%.\n", low * 100.0, high * 100.0);

    println!("--- Commentary (written by the language model) ---\n");
    println!("{}\n", r.commentary);

    println!("--- Risks ---\n");
    println!("Investing in {ticker} carries market, economic and company-specific risks. A forecast from past prices does not anticipate news, and its interval is only as good as the model's fit.");

    println!("\n--- Disclaimer ---\n");
    println!("This report is for research and information only and is not financial advice. Past performance is not indicative of future results. Consult a financial professional before making any investment decision.");
}

/// The smallest and largest weight; `(0.0, 0.0)` for no weights.
fn range(weights: &[f64]) -> (f64, f64) {
    if weights.is_empty() {
        return (0.0, 0.0);
    }
    weights.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &w| (lo.min(w), hi.max(w)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_is_the_smallest_and_largest_weight() {
        assert_eq!(range(&[0.2, 0.5, 0.3]), (0.2, 0.5));
        assert_eq!(range(&[]), (0.0, 0.0));
    }

    #[test]
    fn a_zero_price_is_an_error_not_an_infinite_return() {
        let err = allocate(&[100.0, 0.0, 101.0]).expect_err("a zero close must not divide");
        assert!(err.to_string().contains("start price of 0"), "{err}");
    }
}
