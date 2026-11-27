//! Spans reach an OpenTelemetry collector: a local server plays the
//! collector, and the spans of a real request arrive at `/v1/traces` as
//! OTLP, with their parent links and attributes, scrubbed and keyless.

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use nalufx_api::app::{app, AppState};
use nalufx_api::auth::hash_key;
use nalufx_api::config::ApiConfig;
use nalufx_api::telemetry::{layer, span_exporter, LogFormat};
use serde_json::{json, Value};
use std::time::Duration;
use tower::ServiceExt;
use tracing_subscriber::layer::SubscriberExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const KEY: &str = "otlp-test-key";
const PROVIDER_KEY: &str = "sk-ant-api03-ABCDEFGHIJKLMNOPQRSTUVWXYZ012345";

/// Every span the collector received, from every batch.
async fn received_spans(server: &MockServer) -> (Vec<Value>, String) {
    let mut spans = Vec::new();
    let mut raw = String::new();
    for request in server.received_requests().await.unwrap_or_default() {
        raw.push_str(&String::from_utf8_lossy(&request.body));
        let body: Value = serde_json::from_slice(&request.body).expect("OTLP JSON");
        for resource in body["resourceSpans"].as_array().into_iter().flatten() {
            for scope in resource["scopeSpans"].as_array().into_iter().flatten() {
                spans.extend(scope["spans"].as_array().into_iter().flatten().cloned());
            }
        }
    }
    (spans, raw)
}

/// A span's attribute, as the string or integer OTLP carries.
fn attribute(span: &Value, key: &str) -> Option<String> {
    span["attributes"].as_array()?.iter().find(|a| a["key"] == key).map(|a| {
        let v = &a["value"];
        v["stringValue"].as_str().or_else(|| v["intValue"].as_str()).unwrap_or_default().to_string()
    })
}

#[tokio::test]
async fn a_request_reaches_the_collector_as_linked_scrubbed_spans() {
    let collector = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/traces"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&collector)
        .await;
    let uri = collector.uri();
    let exporter =
        span_exporter(|name| (name == "OTEL_EXPORTER_OTLP_ENDPOINT").then(|| uri.clone()))
            .unwrap()
            .expect("an exporter when the endpoint is set");
    let subscriber =
        tracing_subscriber::registry().with(layer(LogFormat::Json, Some(&exporter)).unwrap());
    let _guard = tracing::subscriber::set_default(subscriber);

    let rows: Vec<Vec<f64>> = (0..40)
        .map(|i| vec![0.01 * (f64::from(i) * 0.7).sin(), 0.02 * f64::from(i).cos()])
        .collect();
    let body = json!({ "assets": ["A", "B"], "returns": rows, "method": "min_variance" });
    let request = Request::post("/v1/allocate")
        .header(header::AUTHORIZATION, format!("Bearer {KEY}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    let service = app(AppState::new(ApiConfig::with_keys(vec![hash_key(KEY)]), None));
    assert_eq!(service.oneshot(request).await.unwrap().status(), StatusCode::OK);
    // A span whose field holds a provider key, to check the scrubbing.
    tracing::info_span!("leaky", note = PROVIDER_KEY).in_scope(|| {});
    assert!(exporter.flush(Duration::from_secs(5)), "the spans were exported in time");

    let (spans, raw) = received_spans(&collector).await;
    assert!(raw.contains("\"nalufx-api\""), "service.name defaults to nalufx-api: {raw}");
    let named = |name: &str| spans.iter().find(|s| s["name"] == name).cloned();
    let request = named("request").expect("a request span");
    let allocate = named("allocate").expect("an allocate span");
    assert_eq!(request["kind"], 2, "the request span is a server span");
    assert_eq!(attribute(&request, "route").as_deref(), Some("/v1/allocate"));
    assert_eq!(attribute(&request, "status").as_deref(), Some("200"));
    let key_id = nalufx_api::telemetry::key_id(&hash_key(KEY));
    assert_eq!(attribute(&request, "key_id"), Some(key_id));
    assert_eq!(allocate["parentSpanId"], request["spanId"], "allocate is a child of request");
    assert_eq!(allocate["traceId"], request["traceId"]);
    assert_eq!(attribute(&allocate, "assets").as_deref(), Some("2"));
    let leaky = named("leaky").expect("the leaky span");
    let note = attribute(&leaky, "note").unwrap_or_default();
    assert!(!note.contains("ABCDEFGHIJ"), "a provider key was exported: {note}");
    assert!(!raw.contains(KEY) && !raw.contains(PROVIDER_KEY), "a key reached the collector");
}

#[test]
fn no_endpoint_means_no_exporter_and_https_is_refused() {
    assert!(span_exporter(|_| None).unwrap().is_none());
    let https = |name: &str| {
        (name == "OTEL_EXPORTER_OTLP_ENDPOINT").then(|| "https://collector.example".to_string())
    };
    assert!(span_exporter(https).is_err(), "rlg-otlp sends plain http to a local collector");
}

/// Sends one allocation with these headers through a fresh collector and
/// returns the exported `request` span.
async fn request_span_with(headers: &[(&str, &str)]) -> Value {
    let collector = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/traces"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&collector)
        .await;
    let uri = collector.uri();
    let exporter =
        span_exporter(|name| (name == "OTEL_EXPORTER_OTLP_ENDPOINT").then(|| uri.clone()))
            .unwrap()
            .unwrap();
    let subscriber =
        tracing_subscriber::registry().with(layer(LogFormat::Json, Some(&exporter)).unwrap());
    let _guard = tracing::subscriber::set_default(subscriber);
    let rows: Vec<Vec<f64>> = (0..20).map(|i| vec![0.01 * f64::from(i).sin(), 0.02]).collect();
    let body = json!({ "assets": ["A", "B"], "returns": rows, "method": "equal_weight" });
    let mut request = Request::post("/v1/allocate")
        .header(header::AUTHORIZATION, format!("Bearer {KEY}"))
        .header(header::CONTENT_TYPE, "application/json");
    for (name, value) in headers {
        request = request.header(*name, *value);
    }
    let service = app(AppState::new(ApiConfig::with_keys(vec![hash_key(KEY)]), None));
    let r = service.oneshot(request.body(Body::from(body.to_string())).unwrap()).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    // The request span closes when the response is dropped.
    drop(r);
    assert!(exporter.flush(Duration::from_secs(5)));
    let (spans, _) = received_spans(&collector).await;
    spans.into_iter().find(|s| s["name"] == "request").expect("a request span")
}

#[tokio::test]
async fn a_traceparent_joins_the_callers_trace() {
    const TRACE: &str = "4bf92f3577b34da6a3ce929d0e0e4736";
    const PARENT: &str = "00f067aa0ba902b7";
    let joined = request_span_with(&[("traceparent", &format!("00-{TRACE}-{PARENT}-01"))]).await;
    assert_eq!(joined["traceId"], TRACE);
    assert_eq!(joined["parentSpanId"], PARENT);
    assert!(attribute(&joined, "traceparent").is_none(), "the header is not an attribute");

    let fresh = request_span_with(&[("traceparent", "00-not-a-valid-header-01")]).await;
    assert_ne!(fresh["traceId"], TRACE);
    assert!(fresh["parentSpanId"].as_str().is_none_or(str::is_empty), "a new root span");
}
