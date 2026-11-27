//! Any bytes as the JSON body of POST /v1/allocate or /v1/forecast, through
//! the real handlers: a problem or a valid answer, never a panic.
#![no_main]

use axum::extract::{Extension, State};
use axum::http::HeaderMap;
use axum::Json;
use libfuzzer_sys::fuzz_target;
use nalufx_api::app::AppState;
use nalufx_api::auth::KeyId;
use nalufx_api::config::ApiConfig;
use nalufx_api::routes::{allocate, forecast, AllocateRequest, ForecastRequest};
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::runtime::{Builder, Runtime};

fn runtime() -> &'static Runtime {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    RUNTIME.get_or_init(|| Builder::new_current_thread().build().expect("a runtime"))
}

/// The state the handlers share; no keys and no provider are needed to
/// call them directly.
fn state() -> Arc<AppState> {
    static STATE: OnceLock<Arc<AppState>> = OnceLock::new();
    STATE.get_or_init(|| Arc::new(AppState::new(ApiConfig::with_keys(Vec::new()), None))).clone()
}

fuzz_target!(|data: &[u8]| {
    if let Ok(request) = serde_json::from_slice::<AllocateRequest>(data) {
        let bounds = request.constraints;
        if let Ok(Json(answer)) = runtime().block_on(allocate(
            State(state()),
            Extension(KeyId("fuzz".into())),
            HeaderMap::new(),
            Json(request),
        )) {
            let total: f64 = answer.weights.iter().sum();
            assert!((total - 1.0).abs() < 1e-6, "weights sum to {total}");
            let (lo, hi) = (bounds.min_weight - 1e-6, bounds.max_weight + 1e-6);
            assert!(
                answer.weights.iter().all(|w| (lo..=hi).contains(w)),
                "a weight outside {bounds:?}: {:?}",
                answer.weights
            );
        }
    }
    if let Ok(request) = serde_json::from_slice::<ForecastRequest>(data) {
        if let Ok(Json(answer)) = runtime().block_on(forecast(Json(request))) {
            let finite = |v: &[f64]| v.iter().all(|x| x.is_finite());
            assert!(finite(&answer.point), "a non-finite forecast");
            assert!(finite(&answer.lower) && finite(&answer.upper), "a non-finite interval");
        }
    }
});
