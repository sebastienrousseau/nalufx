//! The API, driven in process with `tower::ServiceExt::oneshot`.

use async_trait::async_trait;
use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use axum::Router;
use nalufx_api::app::{app, AppState};
use nalufx_api::auth::hash_key;
use nalufx_api::config::ApiConfig;
use nalufx_api::problem::{Problem, PROBLEM_JSON};
use nalufx_llms::error::LlmError;
use nalufx_llms::provider::{ChatRequest, ChatResponse, LlmProvider, TokenUsage};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tower::ServiceExt;

const KEY: &str = "test-key";

const REPORT: &str = r#"{"summary":"Mostly bonds.","rationale":["Bonds are calmer."],"risks":["Rates rise."],"disclaimer":"Research only, not advice."}"#;

/// A provider that answers with fixed text, after waiting on a gate if it
/// has one, counting the calls that reached it.
#[derive(Debug)]
struct Stub {
    text: String,
    delay: Duration,
    gate: Option<Arc<Semaphore>>,
    entered: Arc<AtomicUsize>,
}

impl Stub {
    fn answering(text: &str) -> Self {
        Self { text: text.to_string(), delay: Duration::ZERO, gate: None, entered: Arc::default() }
    }
}

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
        let _ = self.entered.fetch_add(1, Ordering::SeqCst);
        if let Some(gate) = &self.gate {
            let _permit = gate.acquire().await.expect("the gate stays open");
        }
        tokio::time::sleep(self.delay).await;
        Ok(ChatResponse {
            text: self.text.clone(),
            usage: TokenUsage::default(),
            model: "stub-model".into(),
            provider: "stub".into(),
        })
    }
}

fn config() -> ApiConfig {
    ApiConfig::with_keys(vec![hash_key(KEY)])
}

fn service(config: ApiConfig, provider: Option<Stub>) -> Router {
    let provider: Option<Arc<dyn LlmProvider>> = match provider {
        Some(p) => Some(Arc::new(p)),
        None => None,
    };
    app(AppState::new(config, provider))
}

fn post(path: &str, body: &Value) -> Request<Body> {
    Request::post(path)
        .header(header::AUTHORIZATION, format!("Bearer {KEY}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("a request")
}

async fn body(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), 4 * 1024 * 1024).await.expect("a body");
    serde_json::from_slice(&bytes).expect("a JSON body")
}

async fn problem(response: axum::response::Response) -> Problem {
    assert_eq!(response.headers()[header::CONTENT_TYPE], PROBLEM_JSON);
    serde_json::from_value(body(response).await).expect("a problem")
}

fn returns() -> Value {
    let rows: Vec<Vec<f64>> = (0..60)
        .map(|i| {
            let x = f64::from(i);
            vec![0.001 * (x * 0.7).sin(), 0.02 * (x * 1.3).cos()]
        })
        .collect();
    json!(rows)
}

fn history() -> Value {
    json!((0..60)
        .map(|i| 1000.0 + 25.0 * (f64::from(i) * 0.7).sin() + f64::from(i))
        .collect::<Vec<_>>())
}

#[tokio::test]
async fn health_and_readiness_need_no_key() {
    for path in ["/healthz", "/readyz"] {
        let r = service(config(), None)
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(r.status(), StatusCode::OK, "{path}");
        assert_eq!(body(r).await["status"], "ok");
    }
}

#[tokio::test]
async fn readiness_fails_when_the_provider_does_not_answer() {
    let r = service(config(), Some(Stub::answering(REPORT)))
        .oneshot(Request::get("/readyz").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn a_missing_or_wrong_key_is_401() {
    let no_key = Request::post("/v1/forecast").body(Body::from("{}")).unwrap();
    let r = service(config(), None).oneshot(no_key).await.unwrap();
    assert_eq!(r.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(r.headers()[header::WWW_AUTHENTICATE], "Bearer");
    let p = problem(r).await;
    assert_eq!(p.problem_type, "https://nalufx.com/problems/unauthorized");
    assert!(p.request_id.is_some(), "the generated request id is in the problem");

    let wrong = Request::post("/v1/forecast")
        .header(header::AUTHORIZATION, "Bearer nope")
        .body(Body::from("{}"))
        .unwrap();
    assert_eq!(
        service(config(), None).oneshot(wrong).await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn a_valid_key_forecasts_and_allocates() {
    let r = service(config(), None)
        .oneshot(post("/v1/forecast", &json!({ "history": history(), "horizon_days": 3 })))
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(body(r).await["point"].as_array().unwrap().len(), 3);

    let r = service(config(), None)
        .oneshot(post(
            "/v1/allocate",
            &json!({ "assets": ["BONDS", "STOCKS"], "returns": returns(), "method": "hrp" }),
        ))
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let a = body(r).await;
    assert_eq!(a["method"], "hrp");
    let total: f64 = a["weights"].as_array().unwrap().iter().map(|w| w.as_f64().unwrap()).sum();
    assert!((total - 1.0).abs() < 1e-9);
}

#[tokio::test]
async fn engine_and_body_errors_are_problems() {
    let infeasible =
        json!({ "assets": ["A", "B"], "returns": returns(), "constraints": { "max_weight": 0.3 } });
    let r = service(config(), None).oneshot(post("/v1/allocate", &infeasible)).await.unwrap();
    assert_eq!(r.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(problem(r).await.problem_type, "https://nalufx.com/problems/unprocessable");

    let horizon = json!({ "history": history(), "horizon_days": 0 });
    assert_eq!(
        service(config(), None).oneshot(post("/v1/forecast", &horizon)).await.unwrap().status(),
        StatusCode::BAD_REQUEST
    );

    let not_json = Request::post("/v1/forecast")
        .header(header::AUTHORIZATION, format!("Bearer {KEY}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{ nope"))
        .unwrap();
    let r = service(config(), None).oneshot(not_json).await.unwrap();
    assert_eq!(r.status(), StatusCode::BAD_REQUEST);
    assert!(!problem(r).await.detail.is_empty(), "axum's rejection text is kept as the detail");

    let unknown_field = json!({ "history": history(), "colour": "blue" });
    assert_eq!(
        service(config(), None)
            .oneshot(post("/v1/forecast", &unknown_field))
            .await
            .unwrap()
            .status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[tokio::test]
async fn an_oversize_body_is_413() {
    let big = format!(r#"{{"history":[{}1.0]}}"#, "1.0,".repeat(300_000));
    let request = Request::post("/v1/forecast")
        .header(header::AUTHORIZATION, format!("Bearer {KEY}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(big))
        .unwrap();
    let r = service(config(), None).oneshot(request).await.unwrap();
    assert_eq!(r.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(problem(r).await.problem_type, "https://nalufx.com/problems/payload-too-large");
}

#[tokio::test]
async fn a_slow_request_is_408_at_the_timeout() {
    let mut c = config();
    c.timeout = Duration::from_millis(100);
    let slow = Stub { delay: Duration::from_secs(5), ..Stub::answering(REPORT) };
    let r = service(c, Some(slow))
        .oneshot(post("/v1/report", &json!({ "portfolio": "P", "facts": {}, "context": "c" })))
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::REQUEST_TIMEOUT);
    assert_eq!(problem(r).await.problem_type, "https://nalufx.com/problems/timeout");
}

#[tokio::test]
async fn requests_past_the_concurrency_limit_wait() {
    let mut c = config();
    c.concurrency = 2;
    let gate = Arc::new(Semaphore::new(0));
    let entered = Arc::new(AtomicUsize::new(0));
    let stub =
        Stub { gate: Some(gate.clone()), entered: entered.clone(), ..Stub::answering(REPORT) };
    let svc = service(c, Some(stub));
    let report = json!({ "portfolio": "P", "facts": {}, "context": "c" });
    let calls: Vec<_> =
        (0..3).map(|_| tokio::spawn(svc.clone().oneshot(post("/v1/report", &report)))).collect();
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(entered.load(Ordering::SeqCst), 2, "the third request waits for a slot");
    gate.add_permits(3);
    for call in calls {
        assert_eq!(call.await.unwrap().unwrap().status(), StatusCode::OK);
    }
    assert_eq!(entered.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn a_key_over_its_rate_is_429_with_retry_after() {
    let mut c = config();
    c.rate_per_minute = 2;
    let svc = service(c, None);
    let call = || svc.clone().oneshot(post("/v1/forecast", &json!({ "history": history() })));
    assert_eq!(call().await.unwrap().status(), StatusCode::OK);
    assert_eq!(call().await.unwrap().status(), StatusCode::OK);
    let r = call().await.unwrap();
    assert_eq!(r.status(), StatusCode::TOO_MANY_REQUESTS);
    assert!(r.headers()[header::RETRY_AFTER].to_str().unwrap().parse::<u64>().unwrap() >= 1);
    assert_eq!(problem(r).await.problem_type, "https://nalufx.com/problems/rate-limited");
}

#[tokio::test]
async fn reports_need_a_provider_and_a_reply_in_schema() {
    let input = json!({ "portfolio": "P", "facts": { "weights": [0.8, 0.2] }, "context": "c" });
    let r = service(config(), None).oneshot(post("/v1/report", &input)).await.unwrap();
    assert_eq!(r.status(), StatusCode::SERVICE_UNAVAILABLE);

    let r = service(config(), Some(Stub::answering(REPORT)))
        .oneshot(post("/v1/report", &input))
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let report = body(r).await;
    assert_eq!(report["prompt"]["name"], "allocation_report");
    assert_eq!(report["report"]["risks"][0], "Rates rise.");

    let r = service(config(), Some(Stub::answering("not a report")))
        .oneshot(post("/v1/report", &input))
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::BAD_GATEWAY);
}

#[tokio::test]
async fn the_old_endpoint_is_gone_and_points_at_its_successor() {
    let r = service(config(), None)
        .oneshot(Request::post("/predict").body(Body::from("{}")).unwrap())
        .await
        .unwrap();
    assert_eq!(r.status(), StatusCode::GONE);
    assert!(r.headers()[header::LINK].to_str().unwrap().contains("/v1/allocate"));
    assert!(problem(r).await.detail.contains("/v1/allocate"));
}

#[tokio::test]
async fn a_request_id_is_kept_or_made() {
    let given =
        Request::get("/healthz").header("x-request-id", "abc-123").body(Body::empty()).unwrap();
    let r = service(config(), None).oneshot(given).await.unwrap();
    assert_eq!(r.headers()["x-request-id"], "abc-123");

    let r = service(config(), None)
        .oneshot(Request::get("/healthz").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(r.headers()["x-request-id"].len(), 36, "a UUID is generated");

    let missing =
        Request::get("/nowhere").header("x-request-id", "xyz").body(Body::empty()).unwrap();
    let r = service(config(), None).oneshot(missing).await.unwrap();
    assert_eq!(r.status(), StatusCode::NOT_FOUND);
    assert_eq!(problem(r).await.request_id.as_deref(), Some("xyz"));
}

#[tokio::test]
async fn cors_is_denied_unless_the_origin_is_configured() {
    let preflight = |origin: &str| {
        Request::options("/v1/forecast")
            .header(header::ORIGIN, origin)
            .header(header::ACCESS_CONTROL_REQUEST_METHOD, "POST")
            .body(Body::empty())
            .unwrap()
    };
    let r = service(config(), None).oneshot(preflight("https://evil.example")).await.unwrap();
    assert!(r.headers().get(header::ACCESS_CONTROL_ALLOW_ORIGIN).is_none());

    let mut c = config();
    c.cors_origins = vec!["https://app.example".into()];
    let r = service(c, None).oneshot(preflight("https://app.example")).await.unwrap();
    assert_eq!(r.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN], "https://app.example");
}

#[tokio::test]
async fn large_responses_are_compressed_on_request() {
    let request = Request::post("/v1/forecast")
        .header(header::AUTHORIZATION, format!("Bearer {KEY}"))
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ACCEPT_ENCODING, "gzip")
        .body(Body::from(json!({ "history": history(), "horizon_days": 366 }).to_string()))
        .unwrap();
    let r = service(config(), None).oneshot(request).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    assert_eq!(r.headers()[header::CONTENT_ENCODING], "gzip");
}

/// A client that accepts gzip still gets the problem's own detail: the
/// problem layer must see the body before it is compressed.
#[tokio::test]
async fn a_compressed_error_keeps_its_detail() {
    use std::io::Read;
    let request = Request::post("/predict")
        .header(header::ACCEPT_ENCODING, "gzip")
        .body(Body::from("{}"))
        .unwrap();
    let r = service(config(), None).oneshot(request).await.unwrap();
    assert_eq!(r.status(), StatusCode::GONE);
    let gzipped = r.headers().get(header::CONTENT_ENCODING).is_some_and(|v| v == "gzip");
    let bytes = to_bytes(r.into_body(), 1 << 20).await.unwrap();
    let mut text = String::new();
    if gzipped {
        let _ = flate2::read::GzDecoder::new(&bytes[..]).read_to_string(&mut text).unwrap();
    } else {
        text = String::from_utf8(bytes.to_vec()).unwrap();
    }
    let p: Problem = serde_json::from_str(&text).unwrap();
    assert!(p.detail.contains("/v1/allocate"), "{}", p.detail);
}
