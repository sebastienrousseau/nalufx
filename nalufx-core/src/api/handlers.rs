//! Request handlers.
//!
//! Handlers take an already-deserialised request and return a response.
//! Reading and decoding the body is the router's job, which keeps these
//! functions callable from a test without standing up a socket.

use crate::{
    api::response::{self, JsonResponse},
    errors::AllocationError,
    models::cash_flow_dm::{CashFlowResponse, PredictCashFlowRequest},
    utils::calculations::{
        calculate_optimal_allocation, forecast_time_series, DEFAULT_HORIZON_DAYS,
    },
};
use hyper::StatusCode;
use log::error;

/// Forecasts cash flows and allocates across the forecast horizon.
///
/// The cash flow history is forecast [`DEFAULT_HORIZON_DAYS`] ahead with
/// AutoETS, with its 95% interval, and the allocation weights those days
/// by the inverse volatility of the daily-return forecast. Nothing is
/// sent to a model provider.
///
/// # Arguments
///
/// * `request` - The deserialised request body.
///
/// # Returns
///
/// `200` with a [`CashFlowResponse`]; otherwise an RFC 9457 problem: `400`
/// for input that is malformed for the maths (empty, NaN, outliers) and
/// `422` for valid input that cannot be forecast or weighted.
#[must_use]
pub fn predict_cash_flow(request: &PredictCashFlowRequest) -> JsonResponse {
    match predict(request) {
        Ok(body) => response::json(StatusCode::OK, &body),
        Err(err) => {
            error!("predict: {err}");
            response::problem(status_for(&err), err.to_string())
        },
    }
}

/// The forecast and allocation for a request.
fn predict(request: &PredictCashFlowRequest) -> Result<CashFlowResponse, AllocationError> {
    let forecast = forecast_time_series(&request.historical_data, DEFAULT_HORIZON_DAYS)?;
    let optimal_allocation =
        calculate_optimal_allocation(&request.daily_returns, DEFAULT_HORIZON_DAYS)?;
    Ok(CashFlowResponse { forecast, optimal_allocation })
}

/// The HTTP status for an allocation error.
///
/// `400` when the input itself is unusable; `422` when it is well formed
/// but the maths has no answer for it. The match is exhaustive so a new
/// variant has to be given a status here.
#[must_use]
pub const fn status_for(err: &AllocationError) -> StatusCode {
    match err {
        AllocationError::InputMismatch
        | AllocationError::EmptyInput
        | AllocationError::InvalidData
        | AllocationError::OutlierData
        | AllocationError::InvalidHorizon
        | AllocationError::InvalidPrice(_) => StatusCode::BAD_REQUEST,
        AllocationError::ClusteringError(_)
        | AllocationError::ForecastingError(_)
        | AllocationError::ZeroVolatility => StatusCode::UNPROCESSABLE_ENTITY,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(historical_data: Vec<f64>, daily_returns: Vec<f64>) -> PredictCashFlowRequest {
        PredictCashFlowRequest { historical_data, daily_returns }
    }

    #[test]
    fn empty_historical_data_is_a_bad_request() {
        let response = predict_cash_flow(&request(vec![], vec![0.01, -0.01, 0.02]));
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn a_constant_return_series_is_unprocessable() {
        let history: Vec<f64> = (0..30).map(|i| 100.0 + f64::from(i % 5)).collect();
        let response = predict_cash_flow(&request(history, vec![0.01; 30]));
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn every_input_error_is_a_bad_request() {
        for err in [
            AllocationError::InputMismatch,
            AllocationError::EmptyInput,
            AllocationError::InvalidData,
            AllocationError::OutlierData,
            AllocationError::InvalidHorizon,
            AllocationError::InvalidPrice(0.0),
        ] {
            assert_eq!(status_for(&err), StatusCode::BAD_REQUEST, "{err}");
        }
    }

    #[test]
    fn every_maths_error_is_unprocessable() {
        for err in [
            AllocationError::ClusteringError(String::new()),
            AllocationError::ForecastingError(String::new()),
            AllocationError::ZeroVolatility,
        ] {
            assert_eq!(status_for(&err), StatusCode::UNPROCESSABLE_ENTITY, "{err}");
        }
    }
}
