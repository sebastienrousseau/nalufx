//! The routes, their request bodies and their errors.

use crate::app::AppState;
use crate::metrics::LlmCall;
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
use std::time::{Duration, Instant};
use tracing::Instrument;
use utoipa::ToSchema;

/// The longest forecast horizon accepted, in days.
pub const MAX_HORIZON_DAYS: usize = 366;

/// The most history points, or rows of returns, a request may carry.
pub const MAX_POINTS: usize = 10_000;

/// The most assets an allocation may have.
pub const MAX_ASSETS: usize = 500;

/// The most seasonal periods a forecast may have.
pub const MAX_PERIODS: usize = 4;

/// The body of `POST /v1/forecast`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct ForecastRequest {
    /// The history to forecast, oldest first; at most 10,000 points.
    #[schema(max_items = 10000)]
    pub history: Vec<f64>,
    /// Days to forecast, 1 to 366; 6 by default.
    #[serde(default)]
    #[schema(minimum = 1, maximum = 366)]
    pub horizon_days: Option<usize>,
    /// Cycle lengths in the history, such as `[5]` for a weekly business
    /// cycle; empty for none, at most 4.
    #[serde(default)]
    #[schema(max_items = 4)]
    pub seasonal_periods: Vec<usize>,
}

/// The body of `POST /v1/allocate`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct AllocateRequest {
    /// The asset names; at most 500.
    #[schema(max_items = 500)]
    pub assets: Vec<String>,
    /// One row per period, oldest first, with one return per asset; at
    /// most 10,000 rows.
    #[schema(max_items = 10000)]
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
    within("history", request.history.len(), MAX_POINTS)?;
    within("seasonal_periods", request.seasonal_periods.len(), MAX_PERIODS)?;
    let days = request.horizon_days.unwrap_or(DEFAULT_HORIZON_DAYS);
    if !(1..=MAX_HORIZON_DAYS).contains(&days) {
        return Err(Problem::new(
            StatusCode::BAD_REQUEST,
            format!("horizon_days must be 1 to {MAX_HORIZON_DAYS}"),
        ));
    }
    let span = tracing::info_span!(
        "forecast",
        points = request.history.len(),
        days,
        periods = request.seasonal_periods.len()
    );
    let _entered = span.enter();
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
pub async fn allocate(
    State(state): State<Arc<AppState>>,
    Json(request): Json<AllocateRequest>,
) -> Result<Json<Allocation>, Problem> {
    within("assets", request.assets.len(), MAX_ASSETS)?;
    within("returns", request.returns.len(), MAX_POINTS)?;
    let span = tracing::info_span!(
        "allocate",
        method = %request.method,
        assets = request.assets.len(),
        rows = request.returns.len(),
        iterations = tracing::field::Empty,
    );
    let _entered = span.enter();
    let input = AllocationInput {
        returns: returns_matrix(&request.returns, request.assets.len())
            .map_err(allocation_problem)?,
        assets: request.assets.clone(),
        constraints: request.constraints,
        estimator: request.estimator,
    };
    let method = by_name(&request.method, request.risk_aversion).map_err(allocation_problem)?;
    let started = Instant::now();
    let result = method.allocate(&input);
    let solver_failed = matches!(result, Err(AllocationError::SolverFailed(_)));
    state.metrics.allocation(method.name(), started.elapsed(), solver_failed);
    let allocation = result.map_err(allocation_problem)?;
    let _ = span.record("iterations", allocation.diagnostics.iterations);
    Ok(Json(allocation))
}

/// Has the configured language model write a schema-checked report on
/// computed figures. It writes words only.
#[utoipa::path(post, path = "/v1/report", request_body = ReportInput, security(("api_key" = [])),
    responses(
        (status = 200, description = "The validated report, with the prompt, provider and model that wrote it", body = Report),
        (status = 400, description = "Unusable input, such as a context over 2,000 characters", body = Problem, content_type = "application/problem+json"),
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
        Problem::new(StatusCode::SERVICE_UNAVAILABLE, "no language model is configured")
    })?;
    let span = tracing::info_span!(
        "report",
        provider = provider.name(),
        model = provider.model(),
        prompt_version = tracing::field::Empty,
        prompt_tokens = tracing::field::Empty,
        completion_tokens = tracing::field::Empty,
    );
    let started = Instant::now();
    let result = report::generate(provider.as_ref(), &input).instrument(span.clone()).await;
    let usage = result.as_ref().map(|r| r.usage).unwrap_or_default();
    state.metrics.llm(&LlmCall {
        provider: provider.name(),
        model: provider.model(),
        outcome: llm_outcome(result.as_ref().err()),
        prompt_tokens: usage.input_tokens,
        completion_tokens: usage.output_tokens,
        took: started.elapsed(),
    });
    let report = result.map_err(llm_problem)?;
    let _ = span.record("prompt_version", report.prompt.version);
    let _ = span.record("prompt_tokens", usage.input_tokens);
    let _ = span.record("completion_tokens", usage.output_tokens);
    Ok(Json(report))
}

/// A report request's outcome for its metric: `ok`, or the kind of
/// failure.
const fn llm_outcome(err: Option<&LlmError>) -> &'static str {
    match err {
        None => "ok",
        Some(LlmError::Transport { timeout: true, .. }) => "timeout",
        Some(LlmError::Status { status: 429, .. }) => "rate_limited",
        Some(LlmError::Status { .. } | LlmError::Transport { .. }) => "provider_error",
        Some(LlmError::Schema { .. } | LlmError::ParseResponse { .. }) => "bad_reply",
        Some(LlmError::Input(_)) => "bad_input",
        Some(LlmError::MissingApiKey { .. } | LlmError::Config(_)) => "not_configured",
    }
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
            // The error names the provider's address, which an open route
            // must not show; the log keeps it for the operator.
            tracing::warn!(provider = provider.name(), error = %e, "readiness probe failed");
            return Err(Problem::new(
                StatusCode::SERVICE_UNAVAILABLE,
                format!("{} does not answer", provider.name()),
            ));
        }
    }
    Ok(Json(Health { status: "ok".to_string() }))
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

/// A 400 if a list is longer than its limit.
fn within(field: &str, len: usize, max: usize) -> Result<(), Problem> {
    if len > max {
        return Err(Problem::new(
            StatusCode::BAD_REQUEST,
            format!("{field} has {len} items; the most is {max}"),
        ));
    }
    Ok(())
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
        LlmError::Input(_) => (StatusCode::BAD_REQUEST, None),
        LlmError::MissingApiKey { .. } | LlmError::Config(_) => {
            (StatusCode::SERVICE_UNAVAILABLE, None)
        },
        LlmError::Transport { .. }
        | LlmError::Status { .. }
        | LlmError::ParseResponse { .. }
        | LlmError::Schema { .. } => (StatusCode::BAD_GATEWAY, None),
    };
    tracing::warn!(provider = err.provider(), status = status.as_u16(), "language model failed");
    let problem = Problem::new(status, public_detail(&err));
    match retry {
        Some(seconds) => problem.retry_after(seconds),
        None => problem,
    }
}

/// What a client may read about a provider error: the provider and the
/// kind of failure, never the provider's body or message (they can name
/// the operator's account or echo the prompt) nor the server's settings.
/// An input error describes the caller's own request, so it stays whole.
fn public_detail(err: &LlmError) -> String {
    match err {
        LlmError::Input(_) => err.to_string(),
        LlmError::Status { provider, status, .. } => format!("{provider} returned HTTP {status}"),
        LlmError::Transport { provider, timeout: true, .. } => format!("{provider} timed out"),
        LlmError::Transport { provider, .. } => format!("{provider} could not be reached"),
        LlmError::ParseResponse { provider, .. } => {
            format!("{provider} replied in an unexpected format")
        },
        LlmError::Schema { provider, .. } => {
            format!("{provider}'s reply did not match the report schema")
        },
        LlmError::MissingApiKey { .. } | LlmError::Config(_) => {
            "the language model is not available".to_string()
        },
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
        assert_eq!(llm_problem(LlmError::Input(String::new())).status, 400);
    }

    #[test]
    fn a_problem_never_carries_what_the_provider_sent() {
        let private = "org-7Hq2 quota exceeded; your prompt was: secret";
        let errors = [
            LlmError::Status {
                provider: "p",
                status: 500,
                retry_after: None,
                body: private.into(),
            },
            LlmError::Transport { provider: "p", message: private.into(), timeout: false },
            LlmError::ParseResponse { provider: "p", message: private.into() },
            LlmError::Schema { provider: "p", schema: "s", message: private.into() },
            LlmError::Config(private.into()),
            LlmError::MissingApiKey { provider: "p", variable: "P_SECRET_KEY_VAR" },
        ];
        for err in errors {
            let detail = llm_problem(err).detail;
            assert!(!detail.contains("org-7Hq2") && !detail.contains("secret"), "{detail}");
            assert!(!detail.contains("P_SECRET_KEY_VAR"), "{detail}");
        }
        let input = llm_problem(LlmError::Input("context has 3000 characters".into()));
        assert!(input.detail.contains("3000 characters"), "the caller's own input error stays");
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
