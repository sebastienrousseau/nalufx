//! # Performance Measurement Example
//!
//! This example demonstrates how to measure the performance of a stock or portfolio using a Gain/Loss Analysis.
//! It fetches historical market data, calculates the initial market value, net investment, final market value,
//! and capital gain/loss based on OpenAI's analysis of the market at a given date.
//!
//! Usage:
//! 1. Run the code using `cargo run --example performance_measurement`.
//! 2. Enter the ticker symbol for the desired stock or portfolio when prompted.
//! 3. Enter the initial investment amount when prompted.
//! 4. Enter the start date (YYYY-MM-DD) for the analysis period when prompted.
//! 5. Enter the end date (YYYY-MM-DD) for the analysis period when prompted.
//! 6. The code will fetch historical data, perform analysis, and generate a Gain/Loss Analysis report.
//!

use chrono::{DateTime, Utc};
use log::error;
use nalufx::{
    api::handlers::{get_openai_api_key, send_openai_request, OpenAIResponse},
    errors::NaluFxError,
    services::fetch_data::fetch_data,
    utils::{currency::format_currency, date::validate_date, input::get_input},
};
use reqwest::Client;
use serde_json::json;

#[derive(Debug)]
struct StockAnalysis {
    ticker: String,
    initial_market_value: f64,
    final_market_value: f64,
    capital_gain_loss: f64,
    percentage_change: f64,
    eps: f64,
    pe_ratio: f64,
    peg_ratio: f64,
    pb_ratio: f64,
    dpr: f64,
    dividend_yield: f64,
}

/// Validates if the input is a positive floating-point number.
fn validate_positive_float(input: &str) -> Result<f64, &str> {
    match input.parse::<f64>() {
        Ok(value) if value > 0.0 => Ok(value),
        _ => Err("Please enter a valid positive number."),
    }
}

/// Validates if the input is a valid ticker symbol (alphanumeric and non-empty).
fn validate_ticker(input: &str) -> Result<&str, &str> {
    if input.chars().all(|c| c.is_alphanumeric()) && !input.is_empty() {
        Ok(input)
    } else {
        Err("Please enter a valid ticker symbol (alphanumeric).")
    }
}

async fn generate_combined_market_analysis_report(
    stocks: Vec<StockAnalysis>,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
) -> Result<String, &'static str> {
    let client = Client::new();
    let api_key = match get_openai_api_key() {
        Ok(key) => key,
        Err(err) => {
            eprintln!("{}", err);
            return Err("Failed to get OpenAI API key");
        }
    };

    let stock_details: Vec<String> = stocks
        .iter()
        .map(|stock| {
            format!(
                "
Ticker: {ticker}
Initial Market Value: {initial_market_value}
Final Market Value: {final_market_value}
Capital Gain/Loss: {capital_gain_loss}
Percentage Change: {percentage_change:.2}%
EPS: {eps}
P/E Ratio: {pe_ratio}
PEG Ratio: {peg_ratio}
P/B Ratio: {pb_ratio}
DPR: {dpr}
Dividend Yield: {dividend_yield}%",
                ticker = stock.ticker,
                initial_market_value = format_currency(stock.initial_market_value),
                final_market_value = format_currency(stock.final_market_value),
                capital_gain_loss = format_currency(stock.capital_gain_loss),
                percentage_change = stock.percentage_change,
                eps = stock.eps,
                pe_ratio = stock.pe_ratio,
                peg_ratio = stock.peg_ratio,
                pb_ratio = stock.pb_ratio,
                dpr = stock.dpr,
                dividend_yield = stock.dividend_yield
            )
        })
        .collect();

    let combined_stock_details = stock_details.join("\n");

    let request_body = json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "system",
                "content": "You are a highly skilled financial analyst working for a reputable investment firm. Your task is to generate a comprehensive market analysis report for a portfolio of stocks. The report should be written in a professional tone, similar to reports published by Bloomberg or other leading financial institutions. Provide detailed data-driven insights, quantitative analysis, and actionable recommendations. Please use the following structure:"
            },
            {
                "role": "user",
                "content": format!(
                    "
1. **Executive Summary:** Provide a concise summary of the key findings and recommendations.

2. **Market Overview:**
    * Analyze the overall market and economic conditions during the analysis period from {start_date} to {end_date}.
    * Discuss relevant macroeconomic factors, industry trends, and geopolitical events that may have impacted the performance of the portfolio.

3. **Portfolio Performance Analysis:**
    * Analyze the performance of the portfolio during the specified period.
    * Compare the performance of individual stocks against benchmarks or stock market indices.
    * Review key stock indicators, including Earnings Per Share (EPS), Price to Earnings (P/E) ratio, Price to Earnings ratio to Growth ratio (PEG), Price to Book Value ratio (P/B), Dividend Payout ratio (DPR), and Dividend Yield.

4. **Forecast and Recommendations:**
    * Based on the analysis, provide a forecast for the future performance of the portfolio in the short-term and long-term.
    * Discuss potential risks and opportunities for the portfolio.
    * Provide actionable recommendations for investors, such as whether to buy, hold, or sell the stocks or adjust the portfolio allocation.

5. **Conclusion:**
    * Summarize the key takeaways from the analysis and reiterate the main recommendations.
    * Discuss any limitations or uncertainties in the analysis and highlight areas for further research or monitoring.

Please ensure that the report is well-structured, easy to understand, and adheres to industry-standard formatting and terminology.

Portfolio Details:
{combined_stock_details}
                ",
                    start_date = start_date.format("%Y-%m-%d"),
                    end_date = end_date.format("%Y-%m-%d"),
                    combined_stock_details = combined_stock_details
                )
            }
        ],
        "max_tokens": 1500,
    });

    let openai_url = "https://api.openai.com/v1/chat/completions";
    let response = send_openai_request(&client, openai_url, &api_key, request_body)
        .await
        .map_err(|_| "Failed to send OpenAI request")?;

    let openai_response: OpenAIResponse =
        serde_json::from_str(&response).map_err(|_| "Failed to parse OpenAI response")?;

    let generated_text = openai_response
        .choices
        .first()
        .ok_or("No content found in response")?
        .message
        .content
        .clone();

    Ok(generated_text)
}

async fn fetch_data_with_logging(
    ticker: &str,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
) -> Result<Vec<f64>, String> {
    match fetch_data(ticker, start_date, end_date).await {
        Ok(data) => Ok(data),
        Err(err) => {
            error!("Failed to fetch data for {}: {}", ticker, err);
            Err(format!("Failed to fetch data for {}: {}", ticker, err))
        }
    }
}

#[tokio::main]
pub(crate) async fn main() -> Result<(), NaluFxError> {
    // Get user input for tickers, initial investment amount, start date, and end date
    let tickers_input =
        get_input("Enter the ticker symbols for the stocks or portfolio (comma-separated):")?;
    let tickers: Vec<&str> = tickers_input.split(',').map(|s| s.trim()).collect();

    for ticker in &tickers {
        if validate_ticker(ticker).is_err() {
            eprintln!("Error: Invalid ticker symbol: {}", ticker);
            return Err(NaluFxError::InvalidOption);
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

    let start_date_input = get_input("Enter the start date (YYYY-MM-DD):")?;
    let start_date = match validate_date(&start_date_input) {
        Ok(date) => date,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(NaluFxError::InvalidOption);
        }
    };

    let end_date_input = get_input("Enter the end date (YYYY-MM-DD):")?;
    let end_date = match validate_date(&end_date_input) {
        Ok(date) => date,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(NaluFxError::InvalidOption);
        }
    };

    let mut overall_initial_value = 0.0;
    let mut overall_final_value = 0.0;
    let individual_investment = initial_investment / tickers.len() as f64;
    let mut stock_analyses = Vec::new();

    for ticker in tickers {
        // Fetch historical market data for the specified stock or portfolio and date range
        let market_data =
            match fetch_data_with_logging(ticker, Some(start_date), Some(end_date)).await {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Error fetching market data for {}: {}", ticker, e);
                    return Err(NaluFxError::InvalidData);
                }
            };

        // Calculate the initial market value
        let initial_market_value = market_data[0] * individual_investment / initial_investment;

        // Calculate the final market value
        let final_market_value =
            *market_data.last().unwrap() * individual_investment / initial_investment;

        // Calculate the capital gain/loss
        let capital_gain_loss = final_market_value - initial_market_value;

        // Calculate percentage change
        let percentage_change =
            ((final_market_value - initial_market_value) / initial_market_value) * 100.0;

        // Dummy values for financial indicators (you need to fetch these from a financial API or database)
        let eps = 5.0;
        let pe_ratio = 20.0;
        let peg_ratio = 1.5;
        let pb_ratio = 3.0;
        let dpr = 0.6;
        let dividend_yield = 2.5;

        stock_analyses.push(StockAnalysis {
            ticker: ticker.to_string(),
            initial_market_value,
            final_market_value,
            capital_gain_loss,
            percentage_change,
            eps,
            pe_ratio,
            peg_ratio,
            pb_ratio,
            dpr,
            dividend_yield,
        });

        // Update the overall values
        overall_initial_value += initial_market_value;
        overall_final_value += final_market_value;
    }

    // Generate the combined market analysis report using OpenAI
    let combined_analysis_report = match generate_combined_market_analysis_report(
        stock_analyses,
        start_date,
        end_date,
    )
    .await
    {
        Ok(report) => report,
        Err(err) => {
            eprintln!("Error generating combined market analysis report: {}", err);
            return Err(NaluFxError::InvalidData);
        }
    };

    // Calculate the overall capital gain/loss
    let overall_capital_gain_loss = overall_final_value - overall_initial_value;
    let overall_percentage_change =
        ((overall_final_value - overall_initial_value) / overall_initial_value) * 100.0;

    // Print the overall portfolio summary
    println!("\n--- Overall Portfolio Summary ---\n");
    println!(
        "Analysis Period: {} to {}",
        start_date.format("%Y-%m-%d"),
        end_date.format("%Y-%m-%d")
    );
    println!(
        "Overall Initial Market Value: {}",
        format_currency(overall_initial_value)
    );
    println!(
        "Overall Final Market Value: {}",
        format_currency(overall_final_value)
    );
    println!(
        "Overall Capital Gain/Loss: {}",
        format_currency(overall_capital_gain_loss)
    );
    println!(
        "Overall Percentage Change: {:.2}%",
        overall_percentage_change
    );

    // Print the combined report
    println!("\n--- Combined Market Analysis Report ---\n");
    println!("{}", combined_analysis_report);

    Ok(())
}
