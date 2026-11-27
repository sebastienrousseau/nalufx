use crate::models::forecast_dm::IntervalForecast;
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

/// The body of a successful `POST /predict` response.
///
/// # Fields
///
/// * `forecast` - The cash flow forecast with its 95% prediction interval.
/// * `optimal_allocation` - One weight per forecast day, each at least 0,
///   summing to 1 (inverse volatility of the daily-return forecast).
///
/// # Examples
///
/// ```
/// use nalufx::models::{cash_flow_dm::CashFlowResponse, forecast_dm::IntervalForecast};
///
/// let response = CashFlowResponse {
///     forecast: IntervalForecast {
///         level: 0.95,
///         point: vec![1.0, 2.0],
///         lower: vec![0.5, 1.2],
///         upper: vec![1.5, 2.8],
///     },
///     optimal_allocation: vec![0.6, 0.4],
/// };
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CashFlowResponse {
    /// The cash flow forecast with its prediction interval.
    pub forecast: IntervalForecast,
    /// One weight per forecast day, summing to 1.
    pub optimal_allocation: Vec<f64>,
}

/// An error response body in the RFC 9457 problem details format,
/// served as `application/problem+json`.
///
/// # Fields
///
/// * `type` - A URI naming the problem type; `about:blank` means the
///   status code says it all.
/// * `title` - The status code's reason phrase.
/// * `status` - The HTTP status code.
/// * `detail` - What went wrong with this request.
///
/// # Examples
///
/// ```
/// use nalufx::models::cash_flow_dm::ProblemDetails;
///
/// let problem = ProblemDetails {
///     problem_type: "about:blank".to_string(),
///     title: "Bad Request".to_string(),
///     status: 400,
///     detail: "Input slices cannot be empty".to_string(),
/// };
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ProblemDetails {
    /// A URI naming the problem type.
    #[serde(rename = "type")]
    pub problem_type: String,
    /// A short summary of the problem type.
    pub title: String,
    /// The HTTP status code.
    pub status: u16,
    /// An explanation specific to this occurrence.
    pub detail: String,
}

/// The body of a `POST /predict` request.
///
/// # Fields
///
/// * `historical_data` - The cash flow history to forecast, oldest first.
/// * `daily_returns` - The daily returns the allocation is weighted by,
///   oldest first.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PredictCashFlowRequest {
    /// The cash flow history to forecast, oldest first.
    pub historical_data: Vec<f64>,
    /// The daily returns the allocation is weighted by, oldest first.
    pub daily_returns: Vec<f64>,
}
