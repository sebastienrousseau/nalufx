//! # Risk Parity Portfolio Optimization Example
//!
//! This example demonstrates how to create a risk parity portfolio, where the goal is to allocate risk equally across all assets in the portfolio.
//! The code fetches historical performance data for a set of assets, calculates the risk contribution of each asset, and then optimizes the portfolio
//! to achieve equal risk contribution.
//!
//! Definition - Risk Parity:
//! Risk Parity is an investment strategy that aims to allocate risk equally across all assets in a portfolio. The idea behind risk parity is to balance
//! the risk contributions of each asset, rather than focusing on the capital allocation. This approach seeks to reduce the concentration of risk in any
//! single asset or asset class, thereby improving the overall diversification and stability of the portfolio.
//!
//! Usage:
//! 1. Run the code using `cargo run --example risk_parity_portfolio_optimization`.
//! 2. Enter the list of assets (e.g., stock tickers) when prompted.

use nalgebra::{DMatrix, DVector};
use nalufx::errors::NaluFxError;
use nalufx::services::{fetch_data::fetch_data, processing::calculate_daily_returns};
use nalufx::utils::input::get_input;
use ndarray::Array2;
use ndarray_stats::CorrelationExt;
use std::collections::HashMap;

/// Function to optimize the portfolio for risk parity.
///
/// # Arguments
///
/// * `assets` - A vector of asset names (e.g., stock tickers).
/// * `cov_matrix` - The covariance matrix of asset returns.
///
/// # Returns
///
/// A HashMap containing the optimized weights for each asset.
fn optimize_risk_parity(assets: &Vec<&str>, cov_matrix: &Array2<f64>) -> HashMap<String, f64> {
    let num_assets = assets.len();

    // Convert the covariance matrix shape to a tuple
    let cov_matrix_shape = (cov_matrix.nrows(), cov_matrix.ncols());

    // Check if the covariance matrix has the expected shape
    if cov_matrix_shape != (num_assets, num_assets) {
        panic!(
            "Covariance matrix shape ({:?}) does not match the number of assets ({})",
            cov_matrix_shape, num_assets
        );
    }

    // Convert covariance matrix to a Vec<f64>
    let cov_matrix_vec = cov_matrix.iter().cloned().collect::<Vec<f64>>();

    // Create DMatrix from the covariance matrix Vec<f64>
    let cov_matrix_nalgebra = DMatrix::from_row_slice(num_assets, num_assets, &cov_matrix_vec);

    // Define the objective function for risk parity
    let objective = |weights: &DVector<f64>| {
        let portfolio_var = weights.transpose() * &cov_matrix_nalgebra * weights;
        let portfolio_std_dev = portfolio_var[(0, 0)].sqrt();
        let risk_contributions = &cov_matrix_nalgebra * weights / portfolio_std_dev;
        let mean_risk_contribution = risk_contributions.mean();
        let risk_diffs = risk_contributions.map(|x| x - mean_risk_contribution);
        risk_diffs.norm_squared()
    };

    // Define the initial guess for weights
    let mut weights = DVector::from_element(num_assets, 1.0 / num_assets as f64);

    // Perform optimization using gradient descent
    let mut learning_rate = 0.1;
    let max_iterations = 100;
    let tolerance = 1e-6;

    for _ in 0..max_iterations {
        let grad = numerical_gradient(&objective, &weights);
        let new_weights = &weights - learning_rate * &grad;

        // Normalize weights to sum to 1
        let sum_weights = new_weights.sum();
        let normalized_weights = new_weights / sum_weights;

        if (&normalized_weights - &weights).norm() < tolerance {
            weights = normalized_weights;
            break;
        }

        weights = normalized_weights;
        learning_rate *= 0.95;
    }

    // Convert optimized weights to a HashMap
    let mut weights_map = HashMap::new();
    for (i, &asset) in assets.iter().enumerate() {
        let _ = weights_map.insert(asset.to_string(), weights[i]);
    }

    weights_map
}

/// Function to calculate the numerical gradient of a given function.
///
/// # Arguments
///
/// * `f` - The function for which to calculate the numerical gradient.
/// * `x` - The input vector at which to calculate the gradient.
///
/// # Returns
///
/// The numerical gradient of the function at the given input vector.
fn numerical_gradient<F>(f: &F, x: &DVector<f64>) -> DVector<f64>
where
    F: Fn(&DVector<f64>) -> f64,
{
    let eps = 1e-8;
    let mut grad = DVector::zeros(x.nrows());
    let mut x_perturb = x.clone();

    for i in 0..x.nrows() {
        x_perturb[i] += eps;
        let f_plus = f(&x_perturb);
        x_perturb[i] -= 2.0 * eps;
        let f_minus = f(&x_perturb);
        grad[i] = (f_plus - f_minus) / (2.0 * eps);
        x_perturb[i] = x[i];
    }

    grad
}

/// Main function to run the risk parity portfolio optimization.
///
/// This function prompts the user to enter a list of assets, fetches historical performance data for each asset,
/// calculates the covariance matrix, optimizes the portfolio for risk parity, and displays the optimal weights.
///
/// # Returns
///
/// A `Result` indicating the success or failure of the operation. If successful, returns `Ok(())`.
/// If an error occurs, returns an `Err` variant containing the error message.
#[tokio::main]
pub(crate) async fn main() -> Result<(), NaluFxError> {
    // Get user input for list of assets
    let assets_input =
        get_input("Enter the list of assets (comma-separated) - (e.g, SPY, EFA, GLD, IEF):")?;
    let assets: Vec<&str> = assets_input.split(',').map(|s| s.trim()).collect();

    // Fetch historical performance data for each asset
    let mut asset_data = Vec::new();
    let mut min_returns_length = usize::MAX;

    for &asset in &assets {
        match fetch_data(asset, None, None).await {
            Ok(closes) => {
                let daily_returns = calculate_daily_returns(&closes);
                if daily_returns.is_empty() {
                    eprintln!("Insufficient data for asset {}", asset);
                    continue;
                }
                min_returns_length = min_returns_length.min(daily_returns.len());
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

    // Truncate returns to the minimum length across all assets
    let returns_matrix: Vec<Vec<f64>> = asset_data
        .iter()
        .map(|(_, returns)| returns[..min_returns_length].to_vec())
        .collect();

    // Debug: Print the shape of the returns matrix
    println!("Shape of returns_matrix: {:?}", returns_matrix.len());

    let num_assets = returns_matrix.len();
    let num_returns = returns_matrix[0].len();

    // Debug: Print the number of assets and returns
    println!("Number of assets: {}", num_assets);
    println!("Number of returns per asset: {}", num_returns);

    let flattened_returns: Vec<f64> = returns_matrix.into_iter().flatten().collect();

    // Debug: Print the length of the flattened returns vector
    println!("Length of flattened returns: {}", flattened_returns.len());

    let returns_array = Array2::from_shape_vec((num_assets, num_returns), flattened_returns)
        .map_err(|_| NaluFxError::InvalidOption)?;

    // Debug: Print the shape of the returns array
    println!("Shape of returns_array: {:?}", returns_array.dim());

    let cov_matrix = returns_array
        .cov(1.0)
        .map_err(|_| NaluFxError::InvalidOption)?;

    // Optimize the portfolio for risk parity
    let optimal_weights = optimize_risk_parity(&assets, &cov_matrix);

    // Display trailing performance for each asset
    println!("\n--- Trailing Performance ---\n");
    for (asset, returns) in asset_data {
        let cumulative_returns: Vec<f64> = returns
            .iter()
            .scan(1.0, |acc, &x| {
                *acc *= 1.0 + x;
                Some(*acc)
            })
            .collect();
        let total_return = cumulative_returns.last().unwrap_or(&1.0) - 1.0;
        println!(
            "{}: Total Return = {:.2}%, Final Value = ${:.2}",
            asset,
            total_return * 100.0,
            cumulative_returns.last().unwrap_or(&1.0)
        );
    }

    // Display the optimal weights
    println!("\n--- Risk Parity Portfolio Weights ---\n");
    println!(
        "The following figures show an equally weighted portfolio consisting of {} ETFs.",
        assets.join(", ")
    );
    for (asset, weight) in optimal_weights {
        println!("{}: {:.2}%", asset, weight * 100.0);
    }

    Ok(())
}
