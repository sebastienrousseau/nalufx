//! The router and its middleware.

use crate::auth::{authenticate, Limits};
use crate::config::{ApiConfig, MAX_BODY_BYTES};
use crate::metrics::{self, Metrics};
use crate::problem::{normalise, Problem, REQUEST_ID};
use crate::routes;
use crate::telemetry;
use axum::http::{HeaderName, HeaderValue, Method, StatusCode};
use axum::middleware::{from_fn, from_fn_with_state};
use axum::routing::{get, post};
use axum::Router;
use nalufx_llms::provider::LlmProvider;
use std::sync::Arc;
use tower::limit::GlobalConcurrencyLimitLayer;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

/// What every handler can reach.
pub struct AppState {
    /// The configuration.
    pub config: ApiConfig,
    /// The per-key token buckets.
    pub limits: Limits,
    /// The language model for `/v1/report`, if one is configured.
    pub provider: Option<Arc<dyn LlmProvider>>,
    /// The client `/readyz` probes the provider's endpoint with.
    pub probe: reqwest::Client,
    /// The Prometheus metrics.
    pub metrics: Metrics,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("config", &self.config)
            .field("provider", &self.provider.as_ref().map(|p| (p.name(), p.model())))
            .finish_non_exhaustive()
    }
}

impl AppState {
    /// State for `config`, with an optional provider.
    #[must_use]
    pub fn new(config: ApiConfig, provider: Option<Arc<dyn LlmProvider>>) -> Self {
        let limits = Limits::per_minute(config.rate_per_minute, config.report_rate_per_minute);
        Self { config, limits, provider, probe: reqwest::Client::new(), metrics: Metrics::new() }
    }
}

/// The application: `/`, `/openapi.json`, `/healthz` and `/readyz` open,
/// everything else (`/metrics` included) behind an API key, all of it behind the middleware
/// stack.
///
/// The stack, outermost first: request id (`x-request-id`, generated if
/// absent), metrics, compression, problem details for every error, tracing,
/// timeout (408), concurrency limit (excess requests wait), request body
/// limit (413), CORS (denied unless origins are configured).
pub fn app(state: AppState) -> Router {
    let state = Arc::new(state);
    let protected = Router::new()
        .route("/v1/forecast", post(routes::forecast))
        .route("/v1/allocate", post(routes::allocate))
        .route("/v1/report", post(routes::report))
        .route("/metrics", get(metrics::serve))
        .route_layer(from_fn_with_state(state.clone(), authenticate));
    let open = Router::new()
        .route("/", get(routes::index))
        .route("/openapi.json", get(routes::openapi_json))
        .route("/healthz", get(routes::healthz))
        .route("/readyz", get(routes::readyz));
    let request_id = HeaderName::from_static(REQUEST_ID);
    let middleware = ServiceBuilder::new()
        .layer(SetRequestIdLayer::new(request_id.clone(), MakeRequestUuid))
        .layer(PropagateRequestIdLayer::new(request_id))
        .layer(from_fn_with_state(state.clone(), metrics::track))
        // Outside the problem layer, so that layer reads bodies before
        // they are compressed.
        .layer(CompressionLayer::new())
        .layer(from_fn(normalise))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(telemetry::request_span)
                .on_response(telemetry::on_response),
        )
        .layer(TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, state.config.timeout))
        .layer(GlobalConcurrencyLimitLayer::new(state.config.concurrency))
        .layer(RequestBodyLimitLayer::new(MAX_BODY_BYTES))
        .layer(cors(&state.config.cors_origins));
    protected.merge(open).fallback(not_found).with_state(state).layer(middleware)
}

/// CORS for the configured origins; with none, no cross-origin request is
/// allowed.
fn cors(origins: &[String]) -> CorsLayer {
    let allowed: Vec<HeaderValue> = origins.iter().filter_map(|o| o.parse().ok()).collect();
    CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([axum::http::header::AUTHORIZATION, axum::http::header::CONTENT_TYPE])
}

/// Any other path.
async fn not_found() -> Problem {
    Problem::new(StatusCode::NOT_FOUND, "no such route")
}
