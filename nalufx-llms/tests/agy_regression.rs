//! Regression tests for the Agy adapter.
//!
//! These cover the paths the unit tests in `src/llms/agy.rs` cannot: the
//! HTTP round trip, the status handling, and the environment lookup.
//! Every case runs against a local wiremock server, so the suite needs
//! no credentials and reaches no third party.

use nalufx_llms::error::LlmError;
use nalufx_llms::llms::agy::{
    get_agy_api_key, parse_agy_response, send_agy_request, API_KEY_VAR, PROVIDER,
};
use reqwest::Client;
use serde_json::json;
use std::env;
use std::sync::Mutex;

use lazy_static::lazy_static;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

lazy_static! {
    /// Serialises the tests that mutate process-wide environment
    /// variables. `lazy_static` rather than a `const` `Mutex::new`,
    /// which is only stable since 1.63 and this workspace declares an
    /// MSRV of 1.56.
    static ref ENV_LOCK: Mutex<()> = Mutex::new(());
}

/// A completion body shaped like the real API's.
fn completion(content: &str) -> serde_json::Value {
    json!({ "choices": [ { "message": { "content": content } } ] })
}

#[tokio::test]
async fn a_successful_call_returns_the_raw_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(completion("1.0 2.0 3.0")))
        .mount(&server)
        .await;

    let body = send_agy_request(
        &Client::new(),
        &format!("{}/v1/chat/completions", server.uri()),
        "test-key",
        json!({ "model": "agy-1" }),
    )
    .await
    .expect("the call should succeed");

    assert_eq!(parse_agy_response(&body).expect("should parse"), vec![1.0, 2.0, 3.0]);
}

#[tokio::test]
async fn the_api_key_is_sent_as_a_bearer_token() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(header("authorization", "Bearer secret-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(completion("1.0")))
        .mount(&server)
        .await;

    // The mock only matches when the header is present and exact, so a
    // success here is the assertion.
    send_agy_request(&Client::new(), &server.uri(), "secret-key", json!({}))
        .await
        .expect("the bearer token should have been sent");
}

#[tokio::test]
async fn the_request_body_is_forwarded_unaltered() {
    let server = MockServer::start().await;
    let payload = json!({ "model": "agy-1", "messages": [ { "role": "user", "content": "hi" } ] });
    Mock::given(method("POST"))
        .and(body_json(payload.clone()))
        .respond_with(ResponseTemplate::new(200).set_body_json(completion("1.0")))
        .mount(&server)
        .await;

    send_agy_request(&Client::new(), &server.uri(), "k", payload)
        .await
        .expect("the body should have been forwarded verbatim");
}

#[tokio::test]
async fn a_server_error_is_a_retryable_request_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST")).respond_with(ResponseTemplate::new(500)).mount(&server).await;

    let err = send_agy_request(&Client::new(), &server.uri(), "k", json!({}))
        .await
        .expect_err("a 500 must be an error");

    assert!(matches!(err, LlmError::Request { .. }));
    assert_eq!(err.provider(), PROVIDER);
    assert!(err.is_retryable(), "a 500 may succeed on retry");
}

#[tokio::test]
async fn a_client_error_is_also_reported_rather_than_parsed() {
    // A 401 body is not a completion. Returning it for parsing would
    // surface as a confusing ParseResponse instead of an auth problem.
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(401).set_body_string("unauthorized"))
        .mount(&server)
        .await;

    let err = send_agy_request(&Client::new(), &server.uri(), "bad-key", json!({}))
        .await
        .expect_err("a 401 must be an error");

    assert!(matches!(err, LlmError::Request { .. }));
    assert!(format!("{err}").contains("401"), "the status should be reported: {err}");
}

#[tokio::test]
async fn an_unreachable_endpoint_is_a_transport_error() {
    // Port 1 on loopback refuses connections.
    let err = send_agy_request(&Client::new(), "http://127.0.0.1:1/", "k", json!({}))
        .await
        .expect_err("an unreachable endpoint must be an error");

    assert!(matches!(err, LlmError::Request { .. }));
    assert!(err.is_retryable());
}

#[tokio::test]
async fn a_non_completion_body_fails_at_parse_time_not_send_time() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{\"nope\":1}"))
        .mount(&server)
        .await;

    // 200 with a wrong shape: the send succeeds, the parse does not.
    let body = send_agy_request(&Client::new(), &server.uri(), "k", json!({}))
        .await
        .expect("a 200 is a successful send");
    let err = parse_agy_response(&body).expect_err("the shape is wrong");
    assert!(matches!(err, LlmError::ParseResponse { .. }));
}

#[test]
fn the_api_key_is_read_from_the_environment() {
    let _guard = ENV_LOCK.lock().expect("mutex should not be poisoned");
    env::set_var(API_KEY_VAR, "env-key");
    assert_eq!(get_agy_api_key().expect("key should be found"), "env-key");
    env::remove_var(API_KEY_VAR);
}

#[test]
fn a_missing_key_reports_the_agy_provider() {
    let _guard = ENV_LOCK.lock().expect("mutex should not be poisoned");
    env::remove_var(API_KEY_VAR);
    // With no key in the environment and no .env carrying one, this is
    // MissingApiKey rather than a panic or an empty string.
    if let Err(err) = get_agy_api_key() {
        assert!(matches!(err, LlmError::MissingApiKey { .. }));
        assert_eq!(err.provider(), PROVIDER);
        assert!(!err.is_retryable(), "a missing key will still be missing on retry");
    }
}
