//! What an operator sees: the spans of a request and the metrics. Spans
//! carry ids, routes, statuses and sizes, never a key, a prompt or the
//! investor's words; `/metrics` parses as Prometheus text.

use async_trait::async_trait;
use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use nalufx_api::app::{app, AppState};
use nalufx_api::auth::hash_key;
use nalufx_api::config::ApiConfig;
use nalufx_llms::error::LlmError;
use nalufx_llms::provider::{ChatRequest, ChatResponse, LlmProvider, TokenUsage};
use serde_json::json;
use std::fmt::Write as _;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::{Context, Layer, SubscriberExt};

const KEY: &str = "telemetry-key-that-must-not-appear";
const CONTEXT: &str = "my private retirement goals must not appear";
const REPORT: &str = r#"{"summary":"Mostly bonds.","rationale":["Bonds are calmer."],"risks":["Rates rise."],"disclaimer":"Research only, not advice."}"#;

/// A provider that answers a fixed report and counts 300 and 120 tokens.
struct Stub;

#[async_trait]
impl LlmProvider for Stub {
    fn name(&self) -> &'static str {
        "stub"
    }
    fn model(&self) -> &str {
        "stub-model"
    }
    fn base_url(&self) -> &str {
        "http://127.0.0.1:9"
    }
    async fn complete(&self, _: &ChatRequest) -> Result<ChatResponse, LlmError> {
        Ok(ChatResponse {
            text: REPORT.into(),
            usage: TokenUsage { input_tokens: 300, output_tokens: 120 },
            model: "stub-model".into(),
            provider: "stub".into(),
        })
    }
}

/// Every span and event, as `name field=value ...` lines.
#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<String>>>);

struct Line<'a>(&'a mut String);

impl Visit for Line<'_> {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let _ = write!(self.0, " {}={value:?}", field.name());
    }
}

impl<S: Subscriber> Layer<S> for Capture {
    fn on_new_span(&self, attrs: &Attributes<'_>, _: &Id, _: Context<'_, S>) {
        let mut line = format!("span {}", attrs.metadata().name());
        attrs.record(&mut Line(&mut line));
        self.0.lock().unwrap().push(line);
    }

    fn on_record(&self, _: &Id, values: &Record<'_>, _: Context<'_, S>) {
        let mut line = "record".to_string();
        values.record(&mut Line(&mut line));
        self.0.lock().unwrap().push(line);
    }

    fn on_event(&self, event: &Event<'_>, _: Context<'_, S>) {
        let mut line = format!("event {}", event.metadata().level());
        event.record(&mut Line(&mut line));
        self.0.lock().unwrap().push(line);
    }
}

fn post(path: &str, body: &serde_json::Value) -> Request<Body> {
    Request::post(path)
        .header(header::AUTHORIZATION, format!("Bearer {KEY}"))
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-request-id", "req-telemetry-1")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn service() -> axum::Router {
    let provider: Arc<dyn LlmProvider> = Arc::new(Stub);
    app(AppState::new(ApiConfig::with_keys(vec![hash_key(KEY)]), Some(provider)))
}

#[tokio::test]
async fn a_report_request_leaves_named_spans_and_nothing_private() {
    let capture = Capture::default();
    let _guard =
        tracing::subscriber::set_default(tracing_subscriber::registry().with(capture.clone()));
    let input = json!({ "portfolio": "P", "facts": { "weights": [0.8, 0.2] }, "context": CONTEXT });
    let r = service().oneshot(post("/v1/report", &input)).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);

    let lines = capture.0.lock().unwrap().clone();
    let all = lines.join("\n");
    let request = lines.iter().find(|l| l.starts_with("span request")).expect("a request span");
    for field in ["method=POST", "route=\"/v1/report\"", "request_id=\"req-telemetry-1\""] {
        assert!(request.contains(field), "{field} missing from {request}");
    }
    let key_id = nalufx_api::telemetry::key_id(&hash_key(KEY));
    assert!(all.contains(&format!("key_id=\"{key_id}\"")), "{all}");
    assert!(all.contains("status=200") && all.contains("latency_ms="), "{all}");
    let report = lines.iter().find(|l| l.starts_with("span report")).expect("a report span");
    assert!(report.contains("provider=\"stub\"") && report.contains("model=\"stub-model\""));
    assert!(all.contains("prompt_version=2"), "{all}");
    assert!(all.contains("prompt_tokens=300") && all.contains("completion_tokens=120"), "{all}");
    for private in [KEY, CONTEXT, "Mostly bonds"] {
        assert!(!all.contains(private), "{private:?} recorded:\n{all}");
    }
}

#[tokio::test]
async fn an_allocation_span_carries_its_method_size_and_iterations() {
    let capture = Capture::default();
    let _guard =
        tracing::subscriber::set_default(tracing_subscriber::registry().with(capture.clone()));
    let rows: Vec<Vec<f64>> = (0..40)
        .map(|i| vec![0.01 * (f64::from(i) * 0.7).sin(), 0.02 * f64::from(i).cos()])
        .collect();
    let body = json!({ "assets": ["A", "B"], "returns": rows, "method": "min_variance" });
    let r = service().oneshot(post("/v1/allocate", &body)).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let lines = capture.0.lock().unwrap().clone();
    let span = lines.iter().find(|l| l.starts_with("span allocate")).expect("an allocate span");
    for field in ["method=min_variance", "assets=2", "rows=40"] {
        assert!(span.contains(field), "{field} missing from {span}");
    }
    assert!(lines.iter().any(|l| l.starts_with("record") && l.contains("iterations=")));
}

#[tokio::test]
async fn metrics_need_a_key_and_parse_as_prometheus_text() {
    let svc = service();
    let r =
        svc.clone().oneshot(Request::get("/metrics").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(r.status(), StatusCode::UNAUTHORIZED);

    let rows: Vec<Vec<f64>> = (0..40).map(|i| vec![0.01 * f64::from(i).sin(), 0.02]).collect();
    let body = json!({ "assets": ["A", "B"], "returns": rows, "method": "hrp" });
    assert_eq!(svc.clone().oneshot(post("/v1/allocate", &body)).await.unwrap().status(), 200);
    let input = json!({ "portfolio": "P", "facts": {}, "context": "c" });
    assert_eq!(svc.clone().oneshot(post("/v1/report", &input)).await.unwrap().status(), 200);

    let get = Request::get("/metrics")
        .header(header::AUTHORIZATION, format!("Bearer {KEY}"))
        .body(Body::empty())
        .unwrap();
    let r = svc.oneshot(get).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    assert!(r.headers()[header::CONTENT_TYPE]
        .to_str()
        .unwrap()
        .starts_with("application/openmetrics-text"));
    let text = String::from_utf8(to_bytes(r.into_body(), 1 << 20).await.unwrap().to_vec()).unwrap();
    let parsed = prometheus_parse::Scrape::parse(text.lines().map(|l| Ok(l.to_string())))
        .expect("/metrics parses as Prometheus text");
    let value = |name: &str, label: (&str, &str)| {
        parsed
            .samples
            .iter()
            .find(|s| s.metric == name && s.labels.get(label.0) == Some(label.1))
            .map(|s| match s.value {
                prometheus_parse::Value::Counter(v) | prometheus_parse::Value::Untyped(v) => v,
                _ => f64::NAN,
            })
    };
    assert_eq!(value("http_requests_total", ("route", "/v1/allocate")), Some(1.0), "{text}");
    assert_eq!(value("llm_tokens_total", ("kind", "prompt")), Some(300.0), "{text}");
    assert_eq!(value("llm_tokens_total", ("kind", "completion")), Some(120.0), "{text}");
    assert_eq!(value("llm_requests_total", ("outcome", "ok")), Some(1.0), "{text}");
    assert!(parsed.samples.iter().any(|s| s.metric.starts_with("allocation_duration_seconds")));
    assert!(!text.contains(KEY), "a key in the metrics");
}
