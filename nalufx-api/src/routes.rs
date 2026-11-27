//! The routes, their request bodies and their errors.

use crate::app::AppState;
use crate::problem::Problem;
use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use nalufx_engine::allocate::{by_name, Allocation, AllocationInput, Constraints};
use nalufx_engine::covariance::CovarianceEstimator;
use nalufx_engine::forecast::{forecast_seasonal, forecast_time_series, IntervalForecast};
use nalufx_engine::horizon::DEFAULT_HORIZON_DAYS;
use nalufx_engine::AllocationError;
use nalufx_llms::error::LlmError;
use nalufx_llms::report::{self, Report, ReportInput};
use ndarray::Array2;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use utoipa::ToSchema;

/// The longest forecast horizon accepted, in days.
pub const MAX_HORIZON_DAYS: usize = 366;

/// The body of `POST /v1/forecast`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ForecastRequest {
    /// The history to forecast, oldest first.
    pub history: Vec<f64>,
    /// Days to forecast, 1 to 366; 6 by default.
    #[serde(default)]
    #[schema(minimum = 1, maximum = 366)]
    pub horizon_days: Option<usize>,
    /// Cycle lengths in the history, such as `[5]` for a weekly business
    /// cycle; empty for none.
    #[serde(default)]
    pub seasonal_periods: Vec<usize>,
}

/// The body of `POST /v1/allocate`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct AllocateRequest {
    /// The asset names.
    pub assets: Vec<String>,
    /// One row per period, oldest first, with one return per asset.
    pub returns: Vec<Vec<f64>>,
    /// `equal_weight`, `min_variance` (the default), `mean_variance`,
    /// `risk_parity` or `hrp`.
    #[serde(default = "default_method")]
    pub method: String,
    /// The risk aversion, for `mean_variance` only.
    #[serde(default)]
    pub risk_aversion: Option<f64>,
    /// The weight bounds; no cap and no floor by default.
    #[serde(default)]
    pub constraints: Constraints,
    /// The covariance estimator; `ledoit_wolf` by default.
    #[serde(default)]
    pub estimator: CovarianceEstimator,
}

fn default_method() -> String {
    "min_variance".to_string()
}

/// The body of a health or readiness answer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Health {
    /// `ok`.
    pub status: String,
}

/// What `GET /` answers: where to find the contract and the health
/// routes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Index {
    /// `nalufx-api`.
    pub service: String,
    /// The server's version.
    pub version: String,
    /// The OpenAPI 3.1 contract.
    pub openapi: String,
    /// Answers while the process is up.
    pub healthz: String,
    /// Answers when the server is ready to take requests.
    pub readyz: String,
}

/// The committed `openapi.json`, as served at `GET /openapi.json`. A test
/// keeps it equal to the spec the code generates.
pub const OPENAPI_JSON: &str = include_str!("../openapi.json");

/// Points at the contract and the health routes.
#[utoipa::path(get, path = "/", responses((status = 200, description = "Where the contract and the health routes are", body = Index)))]
pub async fn index() -> Json<Index> {
    Json(Index {
        service: "nalufx-api".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        openapi: "/openapi.json".to_string(),
        healthz: "/healthz".to_string(),
        readyz: "/readyz".to_string(),
    })
}

/// The OpenAPI 3.1 contract, as committed.
#[utoipa::path(get, path = "/openapi.json", responses((status = 200, description = "The OpenAPI 3.1 document", content_type = "application/json")))]
pub async fn openapi_json() -> Response {
    ([(header::CONTENT_TYPE, "application/json")], OPENAPI_JSON).into_response()
}

/// Forecasts a series with its 95% prediction interval.
#[utoipa::path(post, path = "/v1/forecast", request_body = ForecastRequest, security(("api_key" = [])),
    responses(
        (status = 200, description = "The forecast", body = IntervalForecast),
        (status = 400, description = "Unusable input", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "No valid API key", body = Problem, content_type = "application/problem+json"),
        (status = 422, description = "The series cannot be forecast", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "Over the key's rate limit", body = Problem, content_type = "application/problem+json"),
    ))]
pub async fn forecast(
    Json(request): Json<ForecastRequest>,
) -> Result<Json<IntervalForecast>, Problem> {
    let days = request.horizon_days.unwrap_or(DEFAULT_HORIZON_DAYS);
    if !(1..=MAX_HORIZON_DAYS).contains(&days) {
        return Err(Problem::new(
            StatusCode::BAD_REQUEST,
            format!("horizon_days must be 1 to {MAX_HORIZON_DAYS}"),
        ));
    }
    let forecast = if request.seasonal_periods.is_empty() {
        forecast_time_series(&request.history, days)
    } else {
        forecast_seasonal(&request.history, &request.seasonal_periods, days)
    };
    Ok(Json(forecast.map_err(allocation_problem)?))
}

/// Allocates across assets by the named method, within the bounds.
#[utoipa::path(post, path = "/v1/allocate", request_body = AllocateRequest, security(("api_key" = [])),
    responses(
        (status = 200, description = "The allocation, with its method, parameters and diagnostics", body = Allocation),
        (status = 400, description = "Unusable input or an unknown method", body = Problem, content_type = "application/problem+json"),
        (status = 401, description = "No valid API key", body = Problem, content_type = "application/problem+json"),
        (status = 422, description = "No weights meet the bounds, or the solver failed", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "Over the key's rate limit", body = Problem, content_type = "application/problem+json"),
    ))]
pub async fn allocate(Json(request): Json<AllocateRequest>) -> Result<Json<Allocation>, Problem> {
    let input = AllocationInput {
        returns: returns_matrix(&request.returns, request.assets.len())
            .map_err(allocation_problem)?,
        assets: request.assets,
        constraints: request.constraints,
        estimator: request.estimator,
    };
    let method = by_name(&request.method, request.risk_aversion).map_err(allocation_problem)?;
    Ok(Json(method.allocate(&input).map_err(allocation_problem)?))
}

/// Has the configured language model write a schema-checked report on
/// computed figures. It writes words only.
#[utoipa::path(post, path = "/v1/report", request_body = ReportInput, security(("api_key" = [])),
    responses(
        (status = 200, description = "The validated report, with the prompt, provider and model that wrote it", body = Report),
        (status = 401, description = "No valid API key", body = Problem, content_type = "application/problem+json"),
        (status = 429, description = "Over the key's report rate limit", body = Problem, content_type = "application/problem+json"),
        (status = 502, description = "The provider failed or replied off the schema", body = Problem, content_type = "application/problem+json"),
        (status = 503, description = "No language model is configured, or the provider is rate limiting", body = Problem, content_type = "application/problem+json"),
        (status = 504, description = "The provider timed out", body = Problem, content_type = "application/problem+json"),
    ))]
pub async fn report(
    State(state): State<Arc<AppState>>,
    Json(input): Json<ReportInput>,
) -> Result<Json<Report>, Problem> {
    let provider = state.provider.as_ref().ok_or_else(|| {
        Problem::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "no language model is configured: set NALUFX_LLM_PROVIDER and NALUFX_LLM_MODEL",
        )
    })?;
    Ok(Json(report::generate(provider.as_ref(), &input).await.map_err(llm_problem)?))
}

/// Answers while the process is up.
#[utoipa::path(get, path = "/healthz", responses((status = 200, description = "The process is up", body = Health)))]
pub async fn healthz() -> Json<Health> {
    Json(Health { status: "ok".to_string() })
}

/// Answers when the configuration is loaded and the language model's
/// endpoint, if one is configured, answers.
#[utoipa::path(get, path = "/readyz", responses(
    (status = 200, description = "Ready", body = Health),
    (status = 503, description = "The language model's endpoint does not answer", body = Problem, content_type = "application/problem+json"),
))]
pub async fn readyz(State(state): State<Arc<AppState>>) -> Result<Json<Health>, Problem> {
    if let Some(provider) = &state.provider {
        let probe =
            state.probe.get(provider.base_url()).timeout(Duration::from_secs(2)).send().await;
        if let Err(e) = probe {
            return Err(Problem::new(
                StatusCode::SERVICE_UNAVAILABLE,
                format!("{} does not answer: {e}", provider.name()),
            ));
        }
    }
    Ok(Json(Health { status: "ok".to_string() }))
}

/// The 0.0.6 endpoint, gone: it points at its replacement.
#[utoipa::path(post, path = "/predict", responses(
    (status = 410, description = "Removed; use POST /v1/forecast and POST /v1/allocate", body = Problem, content_type = "application/problem+json"),
))]
pub async fn predict_gone() -> Response {
    let mut response = Problem::new(
        StatusCode::GONE,
        "POST /predict was removed in 0.0.8; use POST /v1/allocate (and POST /v1/forecast for the cash flow forecast)",
    )
    .into_response();
    if let Ok(link) = "</v1/allocate>; rel=\"successor-version\"".parse() {
        let _ = response.headers_mut().insert(header::LINK, link);
    }
    response
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

/// An engine error as a problem: 400 when the input is unusable, 422 when
/// the maths has no answer for it. Exhaustive, so a new variant needs a
/// status here.
#[must_use]
pub fn allocation_problem(err: AllocationError) -> Problem {
    let status = match err {
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
    };
    Problem::new(status, err.to_string())
}

/// A provider error as a problem. The caller's request was fine, so
/// these are gateway errors: 504 for a timeout, 503 (with `Retry-After`)
/// when the provider is rate limiting or not configured, 502 otherwise.
#[must_use]
pub fn llm_problem(err: LlmError) -> Problem {
    let (status, retry) = match &err {
        LlmError::Transport { timeout: true, .. } => (StatusCode::GATEWAY_TIMEOUT, None),
        LlmError::Status { status: 429, retry_after, .. } => {
            (StatusCode::SERVICE_UNAVAILABLE, Some(retry_after.map_or(30, |d| d.as_secs().max(1))))
        },
        LlmError::MissingApiKey { .. } | LlmError::Config(_) => {
            (StatusCode::SERVICE_UNAVAILABLE, None)
        },
        LlmError::Transport { .. }
        | LlmError::Status { .. }
        | LlmError::ParseResponse { .. }
        | LlmError::Schema { .. } => (StatusCode::BAD_GATEWAY, None),
    };
    let problem = Problem::new(status, err.to_string());
    match retry {
        Some(seconds) => problem.retry_after(seconds),
        None => problem,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_errors_map_to_400_or_422() {
        assert_eq!(allocation_problem(AllocationError::EmptyInput).status, 400);
        assert_eq!(allocation_problem(AllocationError::UnknownMethod("x".into())).status, 400);
        assert_eq!(allocation_problem(AllocationError::Infeasible("x".into())).status, 422);
        assert_eq!(allocation_problem(AllocationError::ZeroVolatility).status, 422);
    }

    #[test]
    fn provider_errors_map_to_gateway_statuses() {
        let timeout = LlmError::Transport { provider: "p", message: String::new(), timeout: true };
        assert_eq!(llm_problem(timeout).status, 504);
        let limited = LlmError::Status {
            provider: "p",
            status: 429,
            retry_after: Some(Duration::from_secs(7)),
            body: String::new(),
        };
        let p = llm_problem(limited);
        assert_eq!((p.status, p.retry_after), (503, Some(7)));
        let schema = LlmError::Schema { provider: "p", schema: "s", message: String::new() };
        assert_eq!(llm_problem(schema).status, 502);
        assert_eq!(llm_problem(LlmError::Config(String::new())).status, 503);
    }

    #[test]
    fn a_ragged_matrix_is_a_mismatch() {
        assert_eq!(
            returns_matrix(&[vec![1.0, 2.0], vec![3.0]], 2),
            Err(AllocationError::InputMismatch)
        );
        assert_eq!(returns_matrix(&[], 2), Err(AllocationError::EmptyInput));
    }
}
