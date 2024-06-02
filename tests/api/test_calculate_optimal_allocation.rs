#[cfg(test)]
mod tests {
    use nalufx::errors::AllocationError;
    use nalufx::utils::calculations::calculate_optimal_allocation;

    #[test]
    fn test_calculate_optimal_allocation() {
        // Test case 1: Normal input
        let daily_returns = vec![0.02, -0.01, 0.03, 0.01];
        let cash_flows = vec![100.0, 50.0, 75.0, 120.0];
        let market_indices = vec![1000.0, 1010.0, 1005.0, 1015.0];
        let fund_characteristics = vec![0.8, 0.9, 0.85, 0.95];
        let num_days = 5;
        let result = calculate_optimal_allocation(
            &daily_returns,
            &cash_flows,
            &market_indices,
            &fund_characteristics,
            num_days,
        );
        assert!(result.is_ok());
        // Add more assertions based on expected output

        // Test case 2: Empty input
        let empty_returns: Vec<f64> = Vec::new();
        let empty_cash_flows: Vec<f64> = Vec::new();
        let empty_market_indices: Vec<f64> = Vec::new();
        let empty_fund_characteristics: Vec<f64> = Vec::new();
        let result: Result<Vec<f64>, AllocationError> = calculate_optimal_allocation(
            &empty_returns,
            &empty_cash_flows,
            &empty_market_indices,
            &empty_fund_characteristics,
            num_days,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AllocationError::EmptyInput);

        // Test case 3: Input slices with different lengths
        let daily_returns = vec![0.02, -0.01, 0.03];
        let cash_flows = vec![100.0, 50.0];
        let market_indices = vec![1000.0, 1010.0, 1005.0];
        let fund_characteristics = vec![0.8, 0.9];
        let result = calculate_optimal_allocation(
            &daily_returns,
            &cash_flows,
            &market_indices,
            &fund_characteristics,
            num_days,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AllocationError::InputMismatch);

        // Test case 4: Input with missing or invalid data
        let daily_returns = vec![0.02, f64::NAN, 0.03];
        let cash_flows = vec![100.0, 50.0, 75.0];
        let market_indices = vec![1000.0, 1010.0, 1005.0];
        let fund_characteristics = vec![0.8, 0.9, 0.85];
        let result = calculate_optimal_allocation(
            &daily_returns,
            &cash_flows,
            &market_indices,
            &fund_characteristics,
            num_days,
        );
        assert_eq!(result.unwrap_err(), AllocationError::InvalidData);

        // Test case 5: Input with outliers
        let daily_returns = vec![0.02, -0.01, 10.0];
        let cash_flows = vec![100.0, 50.0, 75.0];
        let market_indices = vec![1000.0, 1010.0, 1005.0];
        let fund_characteristics = vec![0.8, 0.9, 0.85];
        let result = calculate_optimal_allocation(
            &daily_returns,
            &cash_flows,
            &market_indices,
            &fund_characteristics,
            num_days,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AllocationError::OutlierData);
    }
}
