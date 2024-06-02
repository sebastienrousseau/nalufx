use serde::{Deserialize, Serialize};

/// Represents a request to predict cash flow based on historical data.
///
/// # Fields
///
/// * `historical_data` - A vector of historical cash flow data.
///
/// # Examples
///
/// ```
/// use nalufx::api::models::CashFlowRequest;
///
/// let request = CashFlowRequest {
///     historical_data: vec![1.0, 2.0, 3.0],
/// };
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CashFlowRequest {
    pub historical_data: Vec<f64>,
}

/// Represents a response with predicted cash flow and optimal allocation.
///
/// # Fields
///
/// * `predictions` - A vector of predicted cash flow values.
/// * `optimal_allocation` - A vector of optimal allocation percentages.
///
/// # Examples
///
/// ```
/// use nalufx::api::models::CashFlowResponse;
///
/// let response = CashFlowResponse {
///     predictions: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
///     optimal_allocation: vec![0.5, 0.3, 0.2],
/// };
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CashFlowResponse {
    pub predictions: Vec<f64>,
    pub optimal_allocation: Vec<f64>,
}

/// Represents an error response with an error message.
///
/// # Fields
///
/// * `error` - A string containing the error message.
///
/// # Examples
///
/// ```
/// use nalufx::api::models::ErrorResponse;
///
/// let error_response = ErrorResponse {
///     error: "Invalid historical data".to_string(),
/// };
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ErrorResponse {
    pub error: String,
}
