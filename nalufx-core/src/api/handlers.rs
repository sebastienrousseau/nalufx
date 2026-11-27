//! Request handlers.
//!
//! Handlers take an already-deserialised request and return a response.
//! Reading and decoding the body is the router's job, which keeps these
//! functions callable from a test without standing up a socket.

use crate::{
    api::response::{self, JsonResponse},
    errors::AllocationError,
    models::cash_flow_dm::{CashFlowResponse, PredictCashFlowRequest},
};
use hyper::StatusCode;
use log::error;
use nalufx_engine::allocate::{by_name, AllocationInput};
use nalufx_engine::forecast::{forecast_seasonal, forecast_time_series};
use nalufx_engine::horizon::DEFAULT_HORIZON_DAYS;
use ndarray::Array2;

/// Forecasts cash flows and allocates across assets.
///
/// The cash flow history is forecast [`DEFAULT_HORIZON_DAYS`] ahead with
/// its 95% interval: by AutoETS, or by MSTL when `seasonal_periods` is
/// given. The allocation is the named method's, on the returns matrix,
/// within the constraints. Nothing is sent to a model provider.
///
/// # Arguments
///
/// * `request` - The deserialised request body.
///
/// # Returns
///
/// `200` with a [`CashFlowResponse`]; otherwise an RFC 9457 problem: `400`
/// for input that is unusable (empty, NaN, mismatched, an unknown method
/// or a bad parameter) and `422` for valid input that cannot be forecast,
/// weighted or solved within the constraints.
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
    let forecast = if request.seasonal_periods.is_empty() {
        forecast_time_series(&request.historical_data, DEFAULT_HORIZON_DAYS)?
    } else {
        forecast_seasonal(
            &request.historical_data,
            &request.seasonal_periods,
            DEFAULT_HORIZON_DAYS,
        )?
    };
    let input = AllocationInput {
        assets: request.assets.clone(),
        returns: returns_matrix(&request.returns, request.assets.len())?,
        constraints: request.constraints,
        estimator: request.estimator,
    };
    let allocation = by_name(&request.method, request.risk_aversion)?.allocate(&input)?;
    Ok(CashFlowResponse { forecast, allocation })
}

/// The rows of returns as a matrix with one column per asset.
fn returns_matrix(rows: &[Vec<f64>], assets: usize) -> Result<Array2<f64>, AllocationError> {
    if assets == 0 || rows.is_empty() {
        return Err(AllocationError::EmptyInput);
    }
    if rows.iter().any(|row| row.len() != assets) {
        return Err(AllocationError::InputMismatch);
    }
    Array2::from_shape_vec((rows.len(), assets), rows.concat())
        .map_err(|_| AllocationError::InputMismatch)
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
        | AllocationError::InvalidPrice(_)
        | AllocationError::UnknownMethod(_)
        | AllocationError::InvalidParameter(_) => StatusCode::BAD_REQUEST,
        AllocationError::ClusteringError(_)
        | AllocationError::ForecastingError(_)
        | AllocationError::ZeroVolatility
        | AllocationError::Infeasible(_)
        | AllocationError::SolverFailed(_) => StatusCode::UNPROCESSABLE_ENTITY,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn returns_rows() -> Vec<Vec<f64>> {
        (0..40)
            .map(|i| vec![0.001 * (i as f64 * 0.7).sin(), 0.01 * (i as f64 * 1.3).cos()])
            .collect()
    }

    fn request(historical_data: Vec<f64>) -> PredictCashFlowRequest {
        serde_json::from_value(serde_json::json!({
            "historical_data": historical_data,
            "assets": ["BONDS", "STOCKS"],
            "returns": returns_rows(),
        }))
        .expect("a valid request")
    }

    fn history() -> Vec<f64> {
        (0..40).map(|i| 1000.0 + 25.0 * (i as f64 * 0.7).sin() + i as f64).collect()
    }

    #[test]
    fn the_defaults_are_min_variance_with_ledoit_wolf_and_no_bounds() {
        let r = request(history());
        assert_eq!(r.method, "min_variance");
        assert_eq!(r.estimator, nalufx_engine::covariance::CovarianceEstimator::LedoitWolf);
        assert_eq!(r.constraints, nalufx_engine::allocate::Constraints::default());
        assert!(r.seasonal_periods.is_empty());
    }

    #[test]
    fn a_valid_request_allocates_across_the_assets() {
        assert_eq!(predict_cash_flow(&request(history())).status(), StatusCode::OK);
    }

    #[test]
    fn empty_historical_data_is_a_bad_request() {
        assert_eq!(predict_cash_flow(&request(vec![])).status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn a_ragged_returns_matrix_is_a_bad_request() {
        let mut r = request(history());
        r.returns[3].truncate(1);
        assert_eq!(predict_cash_flow(&r).status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn an_unknown_method_is_a_bad_request() {
        let mut r = request(history());
        r.method = "magic".to_string();
        assert_eq!(predict_cash_flow(&r).status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn infeasible_bounds_are_unprocessable() {
        let mut r = request(history());
        r.constraints.max_weight = 0.3;
        assert_eq!(predict_cash_flow(&r).status(), StatusCode::UNPROCESSABLE_ENTITY);
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
            AllocationError::UnknownMethod(String::new()),
            AllocationError::InvalidParameter(String::new()),
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
            AllocationError::Infeasible(String::new()),
            AllocationError::SolverFailed(String::new()),
        ] {
            assert_eq!(status_for(&err), StatusCode::UNPROCESSABLE_ENTITY, "{err}");
        }
    }
}
