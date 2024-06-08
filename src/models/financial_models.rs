use serde::{Deserialize, Serialize};

/// Represents a request for cash flow predictions, containing historical financial data.
///
/// # Fields
///
/// * `historical_data` - A vector of historical data points (as floating-point numbers).
///
/// # Example
///
/// ```
/// use nalufx::models::financial_models::CashFlowRequest;
///
/// let request = CashFlowRequest {
///     historical_data: vec![1.0, 2.0, 3.0],
/// };
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CashFlowRequest {
    /// A vector of historical data points (as floating-point numbers).
    pub historical_data: Vec<f64>,
}

/// Represents a response containing cash flow predictions and optimal allocations.
///
/// # Fields
///
/// * `predictions` - A vector of predicted cash flow values (as floating-point numbers).
/// * `optimal_allocation` - A vector of optimal allocation percentages (as floating-point numbers).
///
/// # Example
///
/// ```
/// use nalufx::models::financial_models::CashFlowResponse;
///
/// let response = CashFlowResponse {
///     predictions: vec![1.0, 2.0, 3.0],
///     optimal_allocation: vec![0.5, 0.3, 0.2],
/// };
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CashFlowResponse {
    /// A vector of predicted cash flow values (as floating-point numbers).
    pub predictions: Vec<f64>,
    /// A vector of optimal allocation percentages (as floating-point numbers).
    pub optimal_allocation: Vec<f64>,
}

/// Represents an error response with a descriptive error message.
///
/// # Fields
///
/// * `error` - A string containing the error message.
///
/// # Example
///
/// ```
/// use nalufx::models::financial_models::ErrorResponse;
///
/// let error_response = ErrorResponse {
///     error: String::from("An error occurred"),
/// };
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ErrorResponse {
    /// A string containing the error message.
    pub error: String,
}

/// Represents historical data for a specific financial ticker.
///
/// # Fields
///
/// * `ticker` - A string representing the financial ticker symbol.
/// * `data` - A vector of historical data points (as floating-point numbers).
///
/// # Example
///
/// ```
/// use nalufx::models::financial_models::HistoricalData;
///
/// let historical_data = HistoricalData {
///     ticker: String::from("AAPL"),
///     data: vec![150.0, 155.0, 160.0],
/// };
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct HistoricalData {
    /// A string representing the financial ticker symbol.
    pub ticker: String,
    /// A vector of historical data points (as floating-point numbers).
    pub data: Vec<f64>,
}
