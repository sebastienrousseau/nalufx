//! The audit log: a line per allocation that replays to the same weights,
//! a line per report without the investor's words, and no answer when a
//! line cannot be written.

use async_trait::async_trait;
use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use nalufx_api::app::{app, AppState};
use nalufx_api::audit::{AuditEntry, AuditSink, FileSink};
use nalufx_api::auth::hash_key;
use nalufx_api::config::ApiConfig;
use nalufx_api::routes::AllocateRequest;
use nalufx_engine::allocate::{by_name, AllocationInput};
use nalufx_llms::error::LlmError;
use nalufx_llms::provider::{ChatRequest, ChatResponse, LlmProvider, TokenUsage};
use ndarray::Array2;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Arc;
use tower::ServiceExt;

const KEY: &str = "audit-key";
const CONTEXT: &str = "my private retirement goals";
const REPLY: &str = r#"{"summary":"Mostly bonds.","rationale":["Bonds are calmer."],"risks":["Rates rise."],"disclaimer":"Research only, not advice."}"#;

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
            text: REPLY.into(),
            usage: TokenUsage { input_tokens: 10, output_tokens: 5 },
            model: "stub-model".into(),
            provider: "stub".into(),
        })
    }
}

/// A sink that cannot write.
#[derive(Debug)]
struct Broken;

impl AuditSink for Broken {
    fn record(&self, _: &AuditEntry) -> Result<(), String> {
        Err("disk full".into())
    }
}

fn state(sink: Arc<dyn AuditSink>) -> AppState {
    let provider: Arc<dyn LlmProvider> = Arc::new(Stub);
    AppState::new(ApiConfig::with_keys(vec![hash_key(KEY)]), Some(provider)).with_audit(sink)
}

fn post(path: &str, body: &serde_json::Value) -> Request<Body> {
    Request::post(path)
        .header(header::AUTHORIZATION, format!("Bearer {KEY}"))
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-request-id", "req-audit-1")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn lines(path: &Path) -> Vec<AuditEntry> {
    std::fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).expect("an audit line"))
        .collect()
}

fn allocation_body() -> serde_json::Value {
    let rows: Vec<Vec<f64>> = (0..60)
        .map(|i| {
            let x = f64::from(i);
            vec![0.001 * (x * 0.7).sin(), 0.02 * (x * 1.3).cos(), 0.01 * (x * 0.4).sin() + 0.002]
        })
        .collect();
    json!({
        "assets": ["BOND", "EQUITY", "GOLD"],
        "returns": rows,
        "method": "mean_variance",
        "risk_aversion": 3.0,
        "constraints": { "max_weight": 0.7, "min_weight": 0.05 },
        "estimator": "ledoit_wolf"
    })
}

#[tokio::test]
async fn an_audit_line_replays_to_the_same_weights_bit_for_bit() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("audit.jsonl");
    let svc = app(state(Arc::new(FileSink::open(&path).unwrap())));
    let r = svc.oneshot(post("/v1/allocate", &allocation_body())).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let answered: serde_json::Value =
        serde_json::from_slice(&to_bytes(r.into_body(), 1 << 20).await.unwrap()).unwrap();

    let entries = lines(&path);
    assert_eq!(entries.len(), 1);
    let entry = &entries[0];
    assert_eq!((entry.kind.as_str(), entry.request_id.as_str()), ("allocation", "req-audit-1"));
    assert_eq!(entry.key_id, nalufx_api::telemetry::key_id(&hash_key(KEY)));
    assert_eq!(entry.engine_version, env!("CARGO_PKG_VERSION"));

    // The input is what was hashed.
    let request: AllocateRequest = serde_json::from_value(entry.input.clone()).unwrap();
    let canonical = serde_json::to_vec(&request).unwrap();
    assert_eq!(entry.input_sha256, hex::encode(Sha256::digest(&canonical)));

    // Replaying it through the engine gives the same weights, bit for bit.
    let rows = request.returns.len();
    let flat: Vec<f64> = request.returns.concat();
    let input = AllocationInput {
        returns: Array2::from_shape_vec((rows, request.assets.len()), flat).unwrap(),
        assets: request.assets.clone(),
        constraints: request.constraints,
        estimator: request.estimator,
    };
    let replayed =
        by_name(&request.method, request.risk_aversion).unwrap().allocate(&input).unwrap();
    let logged: Vec<f64> = serde_json::from_value(entry.output["weights"].clone()).unwrap();
    let bits = |w: &[f64]| w.iter().map(|x| x.to_bits()).collect::<Vec<u64>>();
    assert_eq!(bits(&replayed.weights), bits(&logged));
    let sent: Vec<f64> = serde_json::from_value(answered["weights"].clone()).unwrap();
    assert_eq!(bits(&sent), bits(&logged), "the log holds what the caller received");
}

#[tokio::test]
async fn a_report_line_holds_hashes_and_counts_not_words() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("audit.jsonl");
    let svc = app(state(Arc::new(FileSink::open(&path).unwrap())));
    let input = json!({ "portfolio": "P", "facts": { "weights": [0.8, 0.2] }, "context": CONTEXT });
    let r = svc.oneshot(post("/v1/report", &input)).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(!text.contains(CONTEXT) && !text.contains("Mostly bonds"), "{text}");
    let entry = &lines(&path)[0];
    assert_eq!(entry.kind, "report");
    assert!(entry.input.is_null());
    assert_eq!(entry.output["prompt"]["version"], 2);
    assert_eq!(entry.output["usage"]["input_tokens"], 10);
    assert_eq!(entry.input_sha256.len(), 64);
}

#[tokio::test]
async fn no_answer_goes_out_without_its_audit_line() {
    let svc = app(state(Arc::new(Broken)));
    let r = svc.clone().oneshot(post("/v1/allocate", &allocation_body())).await.unwrap();
    assert_eq!(r.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = String::from_utf8(to_bytes(r.into_body(), 1 << 20).await.unwrap().to_vec()).unwrap();
    assert!(!body.contains("weights") && !body.contains("disk full"), "{body}");
    let input = json!({ "portfolio": "P", "facts": {}, "context": "c" });
    let r = svc.oneshot(post("/v1/report", &input)).await.unwrap();
    assert_eq!(r.status(), StatusCode::SERVICE_UNAVAILABLE);
}
