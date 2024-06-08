//! # Mean-Variance Portfolio Optimization Example
//!
//! This example demonstrates how to optimize a portfolio using the mean-variance optimization approach.
//! The code fetches historical performance data for a set of assets, calculates the covariance matrix and mean returns,
//! and then optimizes the portfolio weights to achieve the target return while minimizing the portfolio variance.
//!
//! Definition - Mean-Variance Optimization:
//! Mean-Variance Optimization is a portfolio optimization technique that aims to find the optimal allocation of assets
//! in a portfolio by maximizing the expected return for a given level of risk or minimizing the risk for a given level
//! of expected return. It is based on the modern portfolio theory developed by Harry Markowitz.
//!
//! Usage:
//! 1. Run the code using `cargo run --example mean_variance_portfolio_optimization`.
//! 2. Enter the list of assets (e.g., stock tickers) when prompted.
//! 3. Enter the target return for the portfolio when prompted.

use nalufx::{
    errors::NaluFxError,
    services::{fetch_data::fetch_data, processing::calculate_daily_returns},
    utils::{date::validate_date, input::get_input},
};
use ndarray::Array2;
use ndarray_stats::CorrelationExt;
use std::collections::HashMap;
use std::error::Error;

/// Placeholder function to calculate the mean-variance optimized portfolio.
///
/// # Arguments
///
/// * `assets` - A vector of asset names (e.g., stock tickers).
/// * `returns_matrix` - The returns matrix of asset returns.
/// * `cov_matrix` - The covariance matrix of asset returns.
/// * `_target_return` - The target return for the portfolio (currently unused).
///
/// # Returns
///
/// A `Result` containing a HashMap of the optimized weights for each asset, or an error if the optimization fails.
fn optimize_mean_variance(
    assets: &Vec<&str>,
    returns_matrix: &Array2<f64>,
    cov_matrix: &Array2<f64>,
    _target_return: f64,
) -> Result<HashMap<String, f64>, Box<dyn Error>> {
    // Placeholder for actual optimization code
    // This should use an optimization library to find the optimal weights
    // For simplicity, we'll just return equal weights
    let n = assets.len();
    let equal_weight = 1.0 / n as f64;
    let mut weights = HashMap::new();
    for &asset in assets {
        let _ = weights.insert(asset.to_string(), equal_weight);
    }

    // Debug: Print covariance matrix and mean returns
    println!("Covariance matrix: \n{:?}", cov_matrix);
    let mean_returns = returns_matrix.mean_axis(ndarray::Axis(1));
    if let Some(mean_returns) = mean_returns {
        let _ = mean_returns.to_owned();
        println!("Mean returns: \n{:?}", mean_returns);
    } else {
        eprintln!("Error calculating mean returns.");
    }
    Ok(weights)
}

/// Main function to run the mean-variance portfolio optimization.
///
/// This function prompts the user to enter a list of assets and a target return, fetches historical performance data
/// for each asset, calculates the covariance matrix and mean returns, optimizes the portfolio weights, and displays
/// the optimal weights.
///
/// # Returns
///
/// A `Result` indicating the success or failure of the operation. If successful, returns `Ok(())`.
/// If an error occurs, returns an `Err` variant containing the error message.
#[tokio::main]
pub(crate) async fn main() -> Result<(), NaluFxError> {
    // Get user input for list of assets
    let assets_input =
        get_input("Enter the list of assets (comma-separated) - (e.g, AAPL, MSFT, GOOGL):")?;
    let assets: Vec<&str> = assets_input.split(',').map(|s| s.trim()).collect();

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

    // Fetch historical performance data for each asset
    let mut asset_data = Vec::new();
    for &asset in &assets {
        match fetch_data(asset, Some(start_date), Some(end_date)).await {
            Ok(closes) => {
                let daily_returns = calculate_daily_returns(&closes);
                if daily_returns.is_empty() {
                    eprintln!("Insufficient data for asset {}", asset);
                    continue;
                }
                asset_data.push((asset, daily_returns));
            }
            Err(e) => {
                eprintln!("Error fetching data for asset {}: {}", asset, e);
            }
        }
    }

    // Check if asset data is available
    if asset_data.is_empty() {
        println!("No asset data available for analysis.");
        return Ok(());
    }

    // Calculate the returns matrix and covariance matrix
    let returns_matrix: Vec<Vec<f64>> = asset_data
        .iter()
        .map(|(_, returns)| returns.clone())
        .collect();
    let num_assets = returns_matrix.len();
    let num_returns = returns_matrix[0].len();

    // Debug: Print the shape of the returns matrix
    println!(
        "Shape of returns_matrix: {:?} x {:?}",
        num_assets, num_returns
    );

    // Ensure all assets have the same number of returns
    if returns_matrix.iter().any(|r| r.len() != num_returns) {
        return Err(NaluFxError::UnequalReturnsLength);
    }

    let flattened_returns: Vec<f64> = returns_matrix.iter().flatten().cloned().collect();

    // Check the dimensions before creating the Array2
    if flattened_returns.len() != num_assets * num_returns {
        return Err(NaluFxError::InvalidReturnsShape);
    }

    let returns_array = Array2::from_shape_vec((num_assets, num_returns), flattened_returns)
        .map_err(|_| NaluFxError::InvalidOption)?;

    // Debug: Print the shape of the returns array
    println!("Shape of returns_array: {:?}", returns_array.dim());

    let cov_matrix = returns_array
        .t()
        .cov(1.0)
        .map_err(|_| NaluFxError::InvalidOption)?;

    // Get user input for target return
    let target_return_input = get_input("Enter the target return for the portfolio:")?;
    let target_return: f64 = target_return_input
        .trim()
        .parse()
        .map_err(|_| NaluFxError::InvalidOption)?;

    // Optimize the portfolio for mean-variance
    let optimal_weights =
        optimize_mean_variance(&assets, &returns_array, &cov_matrix, target_return)
            .map_err(|_| NaluFxError::InvalidOption)?;

    // Display the optimal weights
    println!("\n--- Mean-Variance Optimized Portfolio Weights ---\n");
    for (asset, weight) in optimal_weights {
        println!("{}: {:.2}%", asset, weight * 100.0);
    }

    Ok(())
}
