//! Prometheus metrics, served at `GET /metrics` behind an API key.
//!
//! | Metric | Labels | |
//! | --- | --- | --- |
//! | `http_requests_total` | route, status | Requests answered. |
//! | `http_request_duration_seconds` | route | Time to answer. |
//! | `allocation_duration_seconds` | method | Time the engine took to allocate. |
//! | `solver_failures_total` | method | Allocations whose solver failed. |
//! | `llm_requests_total` | provider, model, outcome | Report requests to a language model. |
//! | `llm_tokens_total` | provider, model, kind | Tokens used, `kind` being `prompt` or `completion`. |
//! | `llm_request_duration_seconds` | provider, model | Time a report request took. |
//!
//! Labels are few and bounded: a route is one of the served paths or
//! `other`, so an unknown path cannot create a new series.

use crate::app::AppState;
use axum::extract::{Request, State};
use axum::http::header;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use prometheus_client::encoding::text::encode;
use prometheus_client::metrics::counter::Counter;
use prometheus_client::metrics::family::Family;
use prometheus_client::metrics::histogram::{exponential_buckets, Histogram};
use prometheus_client::registry::Registry;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// A metric's labels.
type Labels = Vec<(&'static str, String)>;

/// The served paths; any other is `other` in a label.
const ROUTES: [&str; 9] = [
    "/",
    "/openapi.json",
    "/healthz",
    "/readyz",
    "/metrics",
    "/v1/forecast",
    "/v1/allocate",
    "/v1/report",
    "other",
];

/// The OpenMetrics content type.
pub const CONTENT_TYPE: &str = "application/openmetrics-text; version=1.0.0; charset=utf-8";

/// The route label for a path: the path if it is served, else `other`.
#[must_use]
pub fn route(path: &str) -> &'static str {
    ROUTES.iter().copied().find(|r| *r == path).unwrap_or("other")
}

/// Buckets from 1 ms to about 33 s.
fn seconds() -> Histogram {
    Histogram::new(exponential_buckets(0.001, 2.0, 16))
}

/// Every metric, and the registry that encodes them.
#[derive(Debug)]
pub struct Metrics {
    registry: Registry,
    http_requests: Family<Labels, Counter>,
    http_duration: Family<Labels, Histogram>,
    allocation_duration: Family<Labels, Histogram>,
    solver_failures: Family<Labels, Counter>,
    llm_requests: Family<Labels, Counter>,
    llm_tokens: Family<Labels, Counter>,
    llm_duration: Family<Labels, Histogram>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Metrics {
    /// Every metric at zero, registered.
    #[must_use]
    pub fn new() -> Self {
        let mut m = Self {
            registry: Registry::default(),
            http_requests: Family::default(),
            http_duration: Family::new_with_constructor(seconds),
            allocation_duration: Family::new_with_constructor(seconds),
            solver_failures: Family::default(),
            llm_requests: Family::default(),
            llm_tokens: Family::default(),
            llm_duration: Family::new_with_constructor(seconds),
        };
        let r = &mut m.registry;
        r.register("http_requests", "Requests answered", m.http_requests.clone());
        r.register("http_request_duration_seconds", "Time to answer", m.http_duration.clone());
        r.register(
            "allocation_duration_seconds",
            "Time the engine took to allocate",
            m.allocation_duration.clone(),
        );
        r.register("solver_failures", "Allocations whose solver failed", m.solver_failures.clone());
        r.register("llm_requests", "Report requests to a language model", m.llm_requests.clone());
        r.register("llm_tokens", "Tokens used, prompt or completion", m.llm_tokens.clone());
        r.register(
            "llm_request_duration_seconds",
            "Time a report request took",
            m.llm_duration.clone(),
        );
        m
    }

    /// Counts a request and its duration.
    pub fn request(&self, path: &str, status: u16, took: Duration) {
        let route = route(path).to_string();
        let _ = self
            .http_requests
            .get_or_create(&vec![("route", route.clone()), ("status", status.to_string())])
            .inc();
        self.http_duration.get_or_create(&vec![("route", route)]).observe(took.as_secs_f64());
    }

    /// Records an allocation's duration, and a solver failure.
    pub fn allocation(&self, method: &str, took: Duration, solver_failed: bool) {
        let labels = vec![("method", method.to_string())];
        self.allocation_duration.get_or_create(&labels).observe(took.as_secs_f64());
        if solver_failed {
            let _ = self.solver_failures.get_or_create(&labels).inc();
        }
    }

    /// Records a report request: its outcome (`ok` or an error kind), its
    /// tokens and its duration.
    pub fn llm(&self, call: &LlmCall<'_>) {
        let model =
            || vec![("provider", call.provider.to_string()), ("model", call.model.to_string())];
        let mut requests = model();
        requests.push(("outcome", call.outcome.to_string()));
        let _ = self.llm_requests.get_or_create(&requests).inc();
        for (kind, tokens) in
            [("prompt", call.prompt_tokens), ("completion", call.completion_tokens)]
        {
            let mut labels = model();
            labels.push(("kind", kind.to_string()));
            let _ = self.llm_tokens.get_or_create(&labels).inc_by(u64::from(tokens));
        }
        self.llm_duration.get_or_create(&model()).observe(call.took.as_secs_f64());
    }

    /// The metrics in the OpenMetrics text format.
    ///
    /// # Errors
    ///
    /// If encoding fails, which writing to a `String` does not.
    pub fn encode(&self) -> Result<String, std::fmt::Error> {
        let mut text = String::new();
        encode(&mut text, &self.registry)?;
        Ok(text)
    }
}

/// One report request, for [`Metrics::llm`].
#[derive(Clone, Copy, Debug)]
pub struct LlmCall<'a> {
    /// The provider.
    pub provider: &'a str,
    /// The model.
    pub model: &'a str,
    /// `ok`, or the kind of failure.
    pub outcome: &'a str,
    /// Prompt tokens used.
    pub prompt_tokens: u32,
    /// Completion tokens used.
    pub completion_tokens: u32,
    /// How long it took.
    pub took: Duration,
}

/// Middleware: counts every request by route and status, and times it.
pub async fn track(State(state): State<Arc<AppState>>, request: Request, next: Next) -> Response {
    let path = request.uri().path().to_string();
    let started = Instant::now();
    let response = next.run(request).await;
    state.metrics.request(&path, response.status().as_u16(), started.elapsed());
    response
}

/// `GET /metrics`: every metric, in the OpenMetrics text format.
#[utoipa::path(get, path = "/metrics", security(("api_key" = [])),
    responses(
        (status = 200, description = "Every metric, in the OpenMetrics text format", content_type = "application/openmetrics-text"),
        (status = 401, description = "No valid API key", body = crate::problem::Problem, content_type = "application/problem+json"),
        (status = 429, description = "Over the key's rate limit", body = crate::problem::Problem, content_type = "application/problem+json"),
    ))]
pub async fn serve(State(state): State<Arc<AppState>>) -> Response {
    match state.metrics.encode() {
        Ok(text) => ([(header::CONTENT_TYPE, CONTENT_TYPE)], text).into_response(),
        Err(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_paths_share_one_label() {
        assert_eq!(route("/v1/allocate"), "/v1/allocate");
        assert_eq!(route("/v1/allocate/../../etc"), "other");
        assert_eq!(route("/wp-admin"), "other");
    }

    #[test]
    fn every_metric_is_encoded_once_used() {
        let m = Metrics::new();
        m.request("/v1/allocate", 200, Duration::from_millis(3));
        m.allocation("hrp", Duration::from_millis(2), true);
        m.llm(&LlmCall {
            provider: "anthropic",
            model: "m",
            outcome: "ok",
            prompt_tokens: 120,
            completion_tokens: 80,
            took: Duration::from_millis(900),
        });
        let text = m.encode().unwrap();
        for name in [
            "http_requests_total{route=\"/v1/allocate\",status=\"200\"} 1",
            "http_request_duration_seconds_count{route=\"/v1/allocate\"} 1",
            "allocation_duration_seconds_count{method=\"hrp\"} 1",
            "solver_failures_total{method=\"hrp\"} 1",
            "llm_requests_total{provider=\"anthropic\",model=\"m\",outcome=\"ok\"} 1",
            "llm_tokens_total{provider=\"anthropic\",model=\"m\",kind=\"prompt\"} 120",
            "llm_tokens_total{provider=\"anthropic\",model=\"m\",kind=\"completion\"} 80",
            "llm_request_duration_seconds_count{provider=\"anthropic\",model=\"m\"} 1",
        ] {
            assert!(text.contains(name), "{name} missing from\n{text}");
        }
        assert!(text.ends_with("# EOF\n"));
    }
}
