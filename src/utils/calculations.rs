/// Calculates the optimal allocation based on daily returns and cash flows.
///
/// # Arguments
///
/// * `daily_returns` - A slice of daily returns.
/// * `cash_flows` - A slice of cash flows.
/// * `num_days` - The number of days to generate predictions for.
///
/// # Returns
///
/// A vector of optimal allocations for each day, or an error if input slices have different lengths.
pub fn calculate_optimal_allocation(
    daily_returns: &[f64],
    cash_flows: &[f64],
    num_days: usize,
) -> Result<Vec<f64>, &'static str> {
    // Ensure the input slices have the same length
    if daily_returns.len() != cash_flows.len() {
        return Err("Input slices must have the same length");
    }

    // Early return if the input slices are empty
    if daily_returns.is_empty() || cash_flows.is_empty() {
        return Ok(Vec::new());
    }

    // Calculate averages
    let avg_daily_return = daily_returns.iter().sum::<f64>() / daily_returns.len() as f64;
    let avg_cash_flow = cash_flows.iter().sum::<f64>() / cash_flows.len() as f64;

    // Initialize predictions vector
    let mut predictions = Vec::with_capacity(num_days);

    // Calculate predictions in one pass
    for day in 1..=num_days {
        let predicted_return = avg_daily_return * day as f64;
        let predicted_cash_flow = avg_cash_flow * day as f64;
        predictions.push(predicted_return * predicted_cash_flow);
    }

    // Calculate total prediction to normalize the predictions
    let total_prediction: f64 = predictions.iter().sum();

    // Handle the case where total prediction is zero
    if total_prediction == 0.0 {
        return Ok(Vec::new());
    }

    // Normalize predictions to get the optimal allocations
    Ok(predictions
        .into_iter()
        .map(|p| p / total_prediction)
        .collect())
}
