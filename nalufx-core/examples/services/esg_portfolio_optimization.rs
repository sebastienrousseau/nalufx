//! # ESG-Focused Portfolio Optimization Example
//!
//! This example demonstrates how to optimize a portfolio with a strong emphasis on environmental, social, and governance (ESG) factors.
//! It fetches ESG ratings and historical performance data for a diverse range of socially responsible investments (SRI) and green bonds.
//!
//! The code uses a weighted scoring system to identify the best-performing ESG investments and optimize the portfolio allocation while considering the investor's values and financial objectives.
//! The scoring system assigns equal weights to the ESG rating and the normalized historical returns of each investment.
//!
//! The portfolio allocation is determined based on the weighted scores of the investments. The allocation percentage for each investment is calculated by dividing its score by the total score of all investments.
//!
//! The results are presented in a comprehensive impact report, generated using the OpenAI API. The report highlights the portfolio's ESG performance, carbon footprint reduction, and alignment with the United Nations Sustainable Development Goals (SDGs).
//!
//! # Usage
//!
//! 1. Run the code using `cargo run --example esg_focused_portfolio_optimization`.
//! 2. Enter the investor's values (comma-separated) when prompted. For example: "Environmental sustainability, social responsibility, corporate governance".
//! 3. Enter the investor's financial objectives (comma-separated) when prompted. For example: "Long-term capital appreciation, moderate risk tolerance".
//! 4. Enter the list of ESG investments (comma-separated) when prompted. For example: "ESGU, ESGD, ESGE, SUSL, SUSB, ICLN, PBW, GRID, ACES, SMOG".
//! 5. The code will fetch ESG ratings and historical performance data, optimize the portfolio allocation, and generate an impact report.
//!
//! # Dependencies
//!
//! - `nalufx`: A custom library for handling API requests and data processing.
//! - `reqwest`: A Rust library for making HTTP requests.
//! - `serde` and `serde_json`: Rust libraries for serialization and deserialization of JSON data.
//! - `tokio`: A Rust runtime for writing asynchronous code.
//!
//! # Note
//!
//! - The code assumes the existence of a `nalufx` library with specific modules and functions. Make sure to have the necessary dependencies and configurations in place.
//! - The code uses the OpenAI API to generate the impact report. Make sure to set up the API key and have proper authentication in place.
//! - The example uses dummy ESG ratings for demonstration purposes. In a real-world scenario, you would need to fetch actual ESG ratings from reliable sources.

use nalufx::{
    errors::NaluFxError,
    services::{fetch_data_svc::fetch_data, processing_svc::calculate_daily_returns},
    utils::input::get_input,
};
use nalufx_llms::llms::openai::{get_openai_api_key, send_openai_request};
use nalufx_llms::models::openai_dm::OpenAIResponse;
use serde_json::json;

/// Normalizes a vector of data points to a range between 0 and 1.
///
/// # Arguments
///
/// * `data` - A reference to the vector of data points to normalize.
///
/// # Returns
///
/// A new vector containing the normalized data points.
fn normalize_data(data: &Vec<f64>) -> Vec<f64> {
    let max_value = data.iter().cloned().fold(f64::MIN, f64::max);
    let min_value = data.iter().cloned().fold(f64::MAX, f64::min);
    data.iter().map(|&x| (x - min_value) / (max_value - min_value)).collect()
}

/// Calculates the weighted score of an investment based on its ESG rating and normalized returns.
///
/// # Arguments
///
/// * `esg_rating` - The ESG rating of the investment.
/// * `normalized_returns` - A reference to the vector of normalized returns for the investment.
///
/// # Returns
///
/// The calculated weighted score of the investment.
fn calculate_weighted_score(esg_rating: f64, normalized_returns: &Vec<f64>) -> f64 {
    let performance_score: f64 = normalized_returns.iter().sum();
    // Assuming a 50-50 weight for ESG rating and performance score
    let score = (esg_rating * 0.5) + (performance_score * 0.5);
    score
}

/// The main function that orchestrates the ESG-focused portfolio optimization process.
///
/// # Returns
///
/// A `Result` indicating the success or failure of the operation. Returns `Ok(())` if successful, or an error boxed in `Err` if an error occurs.
#[tokio::main]
pub(crate) async fn main() -> Result<(), NaluFxError> {
    // Get user input for investor's values and financial objectives
    let values_input = get_input("Enter the investor's values (comma-separated) - (e.g, Environmental sustainability, social responsibility, corporate governance):")?;
    let financial_objectives_input = get_input("Enter the investor's financial objectives (comma-separated) - (e.g, Long-term capital appreciation, moderate risk tolerance):")?;

    // Get user input for the list of ESG-focused investments
    let investments_input = get_input("Enter the ESG investments (comma-separated) - (e.g, ESGU, ESGD, ESGE, SUSL, SUSB, ICLN, PBW, GRID, ACES, SMOG):")?;
    let esg_investments: Vec<&str> = investments_input.split(',').map(|s| s.trim()).collect();

    // Fetch ESG ratings and historical performance data for each investment
    let mut esg_data = Vec::new();
    for &investment in &esg_investments {
        match fetch_data(investment, None, None).await {
            Ok(closes) => {
                let daily_returns = calculate_daily_returns(&closes);
                if daily_returns.is_empty() {
                    eprintln!("Insufficient data for investment {}", investment);
                    continue;
                }
                // Fetch ESG rating (dummy data for demonstration purposes)
                let esg_rating = 4.5;
                esg_data.push((investment, daily_returns, esg_rating));
            },
            Err(e) => {
                eprintln!("Error fetching data for investment {}: {}", investment, e);
            },
        }
    }

    // Check if ESG data is available
    if esg_data.is_empty() {
        println!("No ESG data available for analysis.");
        return Ok(());
    }

    // Determine the minimum length of all input slices
    let min_length =
        esg_data.iter().map(|(_, daily_returns, _)| daily_returns.len()).min().unwrap_or(0);

    // Normalize data and calculate weighted scores
    let mut esg_scores = Vec::new();
    for (investment, daily_returns, esg_rating) in &esg_data {
        let normalized_returns = normalize_data(&daily_returns[..min_length].to_vec());
        let score = calculate_weighted_score(*esg_rating, &normalized_returns);
        println!("- Investment: {}, Score: {:.2}", investment, score); // Debug statement
        esg_scores.push((investment, score));
    }

    // Sort investments by score
    esg_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Allocate portfolio based on scores
    let total_score: f64 = esg_scores.iter().map(|(_, score)| score).sum();
    let mut esg_allocations = Vec::new();
    for (investment, score) in esg_scores {
        let allocation = score / total_score;
        println!("- Investment: {}, Allocation: {:.2}%", investment, allocation * 100.0); // Debug statement
        esg_allocations.push((investment, allocation));
    }

    // Generate the impact report using OpenAI
    let client = reqwest::Client::new();
    let api_key = match get_openai_api_key() {
        Ok(key) => key,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(NaluFxError::InvalidData);
        },
    };

    let allocations_str = esg_allocations
        .iter()
        .map(|(investment, allocation)| format!("{}: {:.2}%", investment, allocation * 100.0))
        .collect::<Vec<_>>()
        .join("\n");

    let request_body = json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "system",
                "content": "You are a financial analyst specializing in ESG investing. Generate a comprehensive impact report for an ESG-focused portfolio, highlighting its ESG performance, carbon footprint reduction, and alignment with the United Nations Sustainable Development Goals (SDGs)."
            },
            {
                "role": "user",
                "content": format!("Portfolio Allocations:\n{}\n\nInvestor Values: {}\nFinancial Objectives: {}", allocations_str, values_input, financial_objectives_input)
            }
        ],
        "max_tokens": 1500,
    });

    let openai_url = "https://api.openai.com/v1/chat/completions";
    let response = match send_openai_request(&client, openai_url, &api_key, request_body).await {
        Ok(response) => response,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(NaluFxError::InvalidData);
        },
    };

    let impact_report: OpenAIResponse = serde_json::from_str(&response).unwrap();
    let generated_report = impact_report.choices.first().unwrap().message.content.clone();

    // Print the impact report
    println!("\n--- ESG-Focused Portfolio Impact Report ---\n");
    println!("{}", generated_report);

    Ok(())
}
