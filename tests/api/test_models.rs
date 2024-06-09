#[cfg(test)]
mod tests {
    use nalufx::models::financial_dm::{
        CashFlowRequest, CashFlowResponse, ErrorResponse, HistoricalData,
    };
    use serde_json;

    /// Tests serialization and deserialization of `CashFlowRequest`.
    #[test]
    fn test_cash_flow_request_serialization() {
        let request = CashFlowRequest {
            historical_data: vec![1.0, 2.0, 3.0],
        };
        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: CashFlowRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(request, deserialized);
    }

    /// Tests serialization and deserialization of `CashFlowResponse`.
    #[test]
    fn test_cash_flow_response_serialization() {
        let response = CashFlowResponse {
            predictions: vec![1.0, 2.0, 3.0],
            optimal_allocation: vec![0.5, 0.3, 0.2],
        };
        let serialized = serde_json::to_string(&response).unwrap();
        let deserialized: CashFlowResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(response, deserialized);
    }

    /// Tests serialization and deserialization of `ErrorResponse`.
    #[test]
    fn test_error_response_serialization() {
        let error_response = ErrorResponse {
            error: String::from("An error occurred"),
        };
        let serialized = serde_json::to_string(&error_response).unwrap();
        let deserialized: ErrorResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(error_response, deserialized);
    }

    /// Tests serialization and deserialization of `HistoricalData`.
    #[test]
    fn test_historical_data_serialization() {
        let historical_data = HistoricalData {
            ticker: String::from("AAPL"),
            data: vec![150.0, 155.0, 160.0],
        };
        let serialized = serde_json::to_string(&historical_data).unwrap();
        let deserialized: HistoricalData = serde_json::from_str(&serialized).unwrap();
        assert_eq!(historical_data, deserialized);
    }

    /// Tests default values of `CashFlowRequest`.
    #[test]
    fn test_cash_flow_request_default() {
        let request = CashFlowRequest {
            historical_data: Vec::<f64>::new(),
        };
        assert_eq!(request.historical_data, Vec::<f64>::new());
    }

    /// Tests default values of `CashFlowResponse`.
    #[test]
    fn test_cash_flow_response_default() {
        let response = CashFlowResponse {
            predictions: Vec::<f64>::new(),
            optimal_allocation: Vec::<f64>::new(),
        };
        assert_eq!(response.predictions, Vec::<f64>::new());
        assert_eq!(response.optimal_allocation, Vec::<f64>::new());
    }

    /// Tests default values of `ErrorResponse`.
    #[test]
    fn test_error_response_default() {
        let error_response = ErrorResponse {
            error: String::new(),
        };
        assert_eq!(error_response.error, "");
    }

    /// Tests default values of `HistoricalData`.
    #[test]
    fn test_historical_data_default() {
        let historical_data = HistoricalData {
            ticker: String::new(),
            data: Vec::<f64>::new(),
        };
        assert_eq!(historical_data.ticker, "");
        assert_eq!(historical_data.data, Vec::<f64>::new());
    }

    /// Tests serialization and deserialization of a large `CashFlowRequest`.
    #[test]
    fn test_large_cash_flow_request_serialization() {
        let large_data = vec![1.0; 10000]; // Vector with 10,000 elements
        let request = CashFlowRequest {
            historical_data: large_data.clone(),
        };
        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: CashFlowRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(request.historical_data, deserialized.historical_data);
        assert_eq!(request.historical_data.len(), 10000);
    }

    /// Tests serialization and deserialization of `CashFlowRequest` with boundary float values.
    #[test]
    fn test_boundary_values() {
        let request = CashFlowRequest {
            historical_data: vec![f64::MIN, f64::MAX],
        };
        let serialized = serde_json::to_string(&request).unwrap();
        let deserialized: CashFlowRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(request, deserialized);
    }

    /// Tests deserialization of malformed JSON for `CashFlowRequest`.
    #[test]
    fn test_malformed_json_error() {
        let malformed_json = r#"{"historical_data": [1.0, 2.0, "three"]}"#;
        let result: Result<CashFlowRequest, _> = serde_json::from_str(malformed_json);
        assert!(result.is_err());
    }

    /// Tests serialization and deserialization of `CashFlowRequest` and `CashFlowResponse` with empty vectors.
    #[test]
    fn test_empty_vectors() {
        let request = CashFlowRequest {
            historical_data: vec![],
        };
        let response = CashFlowResponse {
            predictions: vec![],
            optimal_allocation: vec![],
        };
        let serialized_request = serde_json::to_string(&request).unwrap();
        let deserialized_request: CashFlowRequest =
            serde_json::from_str(&serialized_request).unwrap();
        assert_eq!(request, deserialized_request);

        let serialized_response = serde_json::to_string(&response).unwrap();
        let deserialized_response: CashFlowResponse =
            serde_json::from_str(&serialized_response).unwrap();
        assert_eq!(response, deserialized_response);
    }

    /// Tests edge case of `ErrorResponse` with an empty error message.
    #[test]
    fn test_error_response_edge_case() {
        let error_response = ErrorResponse {
            error: String::new(),
        };
        let serialized = serde_json::to_string(&error_response).unwrap();
        let deserialized: ErrorResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(error_response, deserialized);
        assert_eq!(error_response.error, "");
    }

    /// Tests edge case of `HistoricalData` with an empty ticker and data.
    #[test]
    fn test_historical_data_edge_case() {
        let historical_data = HistoricalData {
            ticker: String::new(),
            data: Vec::<f64>::new(),
        };
        let serialized = serde_json::to_string(&historical_data).unwrap();
        let deserialized: HistoricalData = serde_json::from_str(&serialized).unwrap();
        assert_eq!(historical_data, deserialized);
        assert_eq!(historical_data.ticker, "");
        assert_eq!(historical_data.data, Vec::<f64>::new());
    }
}
