pub fn calculate_optimal_allocation(
    daily_returns: &[f64],
    cash_flows: &[f64],
    num_days: usize,
) -> Vec<f64> {
    // Calculate the average daily return
    let avg_daily_return = daily_returns.iter().sum::<f64>() / daily_returns.len() as f64;

    // Calculate the average cash flow
    let avg_cash_flow = cash_flows.iter().sum::<f64>() / cash_flows.len() as f64;

    // Generate predictions based on the average daily return and cash flow
    let mut predictions = Vec::new();
    for i in 1..=num_days {
        let predicted_return = avg_daily_return * i as f64;
        let predicted_cash_flow = avg_cash_flow * i as f64;
        let prediction = predicted_return * predicted_cash_flow;
        predictions.push(prediction);
    }

    // Normalize the predictions to get the optimal allocation
    let total_prediction = predictions.iter().sum::<f64>();
    let optimal_allocation = predictions
        .iter()
        .map(|&p| p / total_prediction)
        .collect::<Vec<_>>();

    optimal_allocation
}
