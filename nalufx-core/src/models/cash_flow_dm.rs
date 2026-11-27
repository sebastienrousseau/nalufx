use crate::models::forecast_dm::IntervalForecast;
use nalufx_engine::allocate::{Allocation, Constraints};
use nalufx_engine::covariance::CovarianceEstimator;
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
/// * `allocation` - The weights across the assets, with the method, its
///   parameters and the solver's diagnostics.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CashFlowResponse {
    /// The cash flow forecast with its prediction interval.
    pub forecast: IntervalForecast,
    /// The allocation across the assets.
    pub allocation: Allocation,
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
/// * `seasonal_periods` - Cycle lengths in the cash flows, such as `[5]`
///   for a weekly business-day cycle; empty (the default) for none.
/// * `assets` - The names of the assets to allocate across.
/// * `returns` - One row per period, oldest first, with one return per asset.
/// * `method` - The allocation method; `min_variance` by default.
/// * `risk_aversion` - For `mean_variance` only.
/// * `constraints` - The weight bounds; no cap and no floor by default.
/// * `estimator` - The covariance estimator; `ledoit_wolf` by default.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PredictCashFlowRequest {
    /// The cash flow history to forecast, oldest first.
    pub historical_data: Vec<f64>,
    /// Cycle lengths in the cash flows; empty for none.
    #[serde(default)]
    pub seasonal_periods: Vec<usize>,
    /// The names of the assets to allocate across.
    pub assets: Vec<String>,
    /// One row per period, oldest first, with one return per asset.
    pub returns: Vec<Vec<f64>>,
    /// The allocation method.
    #[serde(default = "default_method")]
    pub method: String,
    /// The risk aversion, for `mean_variance` only.
    #[serde(default)]
    pub risk_aversion: Option<f64>,
    /// The weight bounds.
    #[serde(default)]
    pub constraints: Constraints,
    /// The covariance estimator.
    #[serde(default)]
    pub estimator: CovarianceEstimator,
}

/// The method `/predict` uses when none is named.
fn default_method() -> String {
    "min_variance".to_string()
}
