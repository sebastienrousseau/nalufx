pub fn calculate_optimal_allocation(predictions: &[f64]) -> Vec<f64> {
    // Implement your allocation logic here
    predictions.iter().map(|&x| x * 0.9).collect()
}
