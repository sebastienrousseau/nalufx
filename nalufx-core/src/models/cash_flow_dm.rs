use serde::{Deserialize, Serialize};

/// Represents a request to predict cash flow based on historical data.
///
/// This struct is used to encapsulate the historical cash flow data
/// which will be used by the prediction service to forecast future cash flows.
///
/// # Fields
///
/// * `historical_data` - A vector of historical cash flow data. Each entry represents
///   the cash flow value at a specific time point. It is expected to be a time-series
///   data in chronological order.
///
/// # Examples
///
/// ```
/// use nalufx::models::cash_flow_dm::CashFlowRequest;
///
/// let request = CashFlowRequest {
///     historical_data: vec![1.0, 2.0, 3.0],
/// };
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CashFlowRequest {
    /// A vector of historical cash flow data.
    ///
    /// Each entry in this vector represents the cash flow value at a specific time point.
    /// The data is expected to be ordered chronologically.
    pub historical_data: Vec<f64>,
}

/// Represents a response with predicted cash flow and optimal allocation.
///
/// This struct is used to provide the output of the prediction service,
/// including the predicted future cash flow values and the optimal allocation
/// percentages for investment or resource allocation.
///
/// # Fields
///
/// * `predictions` - A vector of predicted cash flow values. Each entry represents
///   the predicted cash flow value at a specific future time point.
/// * `optimal_allocation` - A vector of optimal allocation percentages. Each entry
///   represents the percentage of allocation for a specific resource or investment
///   based on the predicted cash flow.
///
/// # Examples
///
/// ```
/// use nalufx::models::cash_flow_dm::CashFlowResponse;
///
/// let response = CashFlowResponse {
///     predictions: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
///   optimal_allocation: vec![0.5, 0.3, 0.2],
/// };
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CashFlowResponse {
    /// A vector of predicted cash flow values.
    ///
    /// Each entry in this vector represents the predicted cash flow value at a specific future time point.
    pub predictions: Vec<f64>,
    /// A vector of optimal allocation percentages.
    ///
    /// Each entry in this vector represents the percentage of allocation for a specific resource or investment
    /// based on the predicted cash flow.
    pub optimal_allocation: Vec<f64>,
}

/// Represents an error response with an error message.
///
/// This struct is used to provide details about any errors that occurred
/// during the processing of a request. The error message gives more information
/// about the nature of the error, which can be used for debugging or informing
/// the user.
///
/// # Fields
///
/// * `error` - A string containing the error message. This provides a human-readable
///   explanation of what went wrong.
///
/// # Examples
///
/// ```
/// use nalufx::models::cash_flow_dm::ErrorResponse;
///
/// let error_response = ErrorResponse {
///     error: "Invalid historical data".to_string(),
/// };
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ErrorResponse {
    /// A string containing the error message.
    ///
    /// This provides a human-readable explanation of what went wrong during the processing of the request.
    pub error: String,
}

/// The body of a `POST /predict` request.
///
/// The four series are supplied alongside the historical data in a single
/// JSON object. They were previously five separate body extractors, which
/// cannot work: an HTTP request has one body, so only the first could ever
/// have been read.
///
/// # Fields
///
/// * `historical_data` - The cash flow history used to build the prompt.
/// * `daily_returns` - Daily returns for each fund under consideration.
/// * `cash_flows` - Observed cash flows for each fund.
/// * `market_indices` - Market index values aligned with the other series.
/// * `fund_characteristics` - Per-fund characteristic scores.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PredictCashFlowRequest {
    /// The cash flow history used to build the prompt.
    pub historical_data: Vec<f64>,
    /// Daily returns for each fund under consideration.
    pub daily_returns: Vec<f64>,
    /// Observed cash flows for each fund.
    pub cash_flows: Vec<f64>,
    /// Market index values aligned with the other series.
    pub market_indices: Vec<f64>,
    /// Per-fund characteristic scores.
    pub fund_characteristics: Vec<f64>,
}
