#[cfg(test)]
mod tests {
    use nalufx::utils::calculations::calculate_optimal_allocation;
    use std::error::Error;

    #[test]
    fn test_calculate_optimal_allocation() {
        // Test case 1: Normal input
        let daily_returns = vec![0.02, -0.01, 0.03, 0.01];
        let cash_flows = vec![100.0, 50.0, 75.0, 120.0];
        let num_days = 5;
        let result = calculate_optimal_allocation(&daily_returns, &cash_flows, num_days);
        assert!(result.is_ok());
        // Add more assertions based on expected output

        // Test case 2: Empty input
        let empty_returns: Vec<f64> = Vec::new();
        let empty_cash_flows: Vec<f64> = Vec::new();
        let result: Result<Vec<f64>, Box<dyn Error>> =
            calculate_optimal_allocation(&empty_returns, &empty_cash_flows, num_days);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Vec::<f64>::new());

        // Test case 3: Input slices with different lengths
        let daily_returns = vec![0.02, -0.01, 0.03];
        let cash_flows = vec![100.0, 50.0];
        let result = calculate_optimal_allocation(&daily_returns, &cash_flows, num_days);
        assert!(result.is_err());
    }
}
