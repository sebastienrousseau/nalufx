//! Tests for the `/predict` endpoint.
//!
//! These drive a real server on an ephemeral port rather than a mock
//! handler. The previous versions of these tests asserted against a
//! `mock_predict_cash_flow` defined in this file, so they passed on
//! hardcoded constants and exercised none of the production path.
//!
//! Every case here is reachable without a network call to the model
//! provider: rejection happens before the provider is contacted.

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;
    use nalufx::{api::server::serve_on, models::cash_flow_dm::ErrorResponse};
    use nalufx_llms::{
        error::LlmError,
        llms::openai::{get_openai_api_key, parse_openai_response, send_openai_request},
    };
    use reqwest::Client;
    use serde_json::json;
    use std::env;
    use std::sync::Mutex;
    use tokio::net::TcpListener;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, ResponseTemplate,
    };

    lazy_static! {
        /// Serialises tests that mutate process-wide environment variables.
        static ref ENV_MUTEX: Mutex<()> = Mutex::new(());
    }

    /// Starts the server on an ephemeral port and returns its base URL.
    async fn spawn_server() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("should bind");
        let addr = listener.local_addr().expect("should have an address");
        drop(tokio::spawn(async move {
            let _ = serve_on(listener).await;
        }));
        format!("http://{addr}")
    }

    #[tokio::test]
    async fn empty_historical_data_is_rejected() {
        let base = spawn_server().await;
        let response = Client::new()
            .post(format!("{base}/predict"))
            .json(&json!({
                "historical_data": [],
                "daily_returns": [0.1], "cash_flows": [1.0],
                "market_indices": [100.0], "fund_characteristics": [0.5],
            }))
            .send()
            .await
            .expect("should reach the server");

        assert_eq!(response.status(), 400);
        let body: ErrorResponse = response.json().await.expect("should be an ErrorResponse");
        assert_eq!(body.error, "Invalid historical data");
    }

    #[tokio::test]
    async fn a_malformed_body_is_rejected() {
        let base = spawn_server().await;
        let response = Client::new()
            .post(format!("{base}/predict"))
            .header("content-type", "application/json")
            .body("not json")
            .send()
            .await
            .expect("should reach the server");

        assert_eq!(response.status(), 400);
        let body: ErrorResponse = response.json().await.expect("should be an ErrorResponse");
        assert_eq!(body.error, "Invalid request body");
    }

    #[tokio::test]
    async fn a_body_missing_a_series_is_rejected() {
        // The endpoint needs all four series; supplying only the history
        // used to be accepted because each series was its own extractor.
        let base = spawn_server().await;
        let response = Client::new()
            .post(format!("{base}/predict"))
            .json(&json!({ "historical_data": [1.0, 2.0, 3.0] }))
            .send()
            .await
            .expect("should reach the server");

        assert_eq!(response.status(), 400);
    }

    #[tokio::test]
    async fn an_unknown_path_is_not_found() {
        let base = spawn_server().await;
        let response = Client::new()
            .post(format!("{base}/nope"))
            .json(&json!({}))
            .send()
            .await
            .expect("should reach the server");

        assert_eq!(response.status(), 404);
    }

    #[tokio::test]
    async fn an_unsupported_method_is_rejected() {
        let base = spawn_server().await;
        let response = Client::new()
            .delete(format!("{base}/predict"))
            .send()
            .await
            .expect("should reach the server");

        assert_eq!(response.status(), 405);
    }

    #[tokio::test]
    async fn the_api_key_is_read_from_the_environment() {
        let _guard = ENV_MUTEX.lock().expect("mutex should not be poisoned");
        env::set_var("OPENAI_API_KEY", "test_api_key");
        assert_eq!(get_openai_api_key().expect("key should be found"), "test_api_key");
        env::remove_var("OPENAI_API_KEY");
    }

    #[tokio::test]
    async fn a_successful_provider_call_returns_the_body() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "choices": [{"message": {"content": "1.0 2.0 3.0"}}]
            })))
            .mount(&mock_server)
            .await;

        let body = send_openai_request(
            &Client::new(),
            &format!("{}/v1/chat/completions", mock_server.uri()),
            "test_api_key",
            json!({"model": "gpt-3.5-turbo"}),
        )
        .await
        .expect("the provider call should succeed");

        assert_eq!(parse_openai_response(&body).expect("should parse"), vec![1.0, 2.0, 3.0]);
    }

    #[tokio::test]
    async fn a_provider_error_status_is_a_retryable_request_error() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/chat/completions"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let err = send_openai_request(
            &Client::new(),
            &format!("{}/v1/chat/completions", mock_server.uri()),
            "test_api_key",
            json!({"model": "gpt-3.5-turbo"}),
        )
        .await
        .expect_err("a 500 from the provider should be an error");

        assert!(matches!(err, LlmError::Request { .. }));
        assert_eq!(err.provider(), "openai");
        assert!(err.is_retryable());
    }

    #[tokio::test]
    async fn an_unexpected_provider_shape_is_a_parse_error() {
        let result = parse_openai_response(r#"{"unexpected": "structure"}"#);
        assert!(matches!(result, Err(LlmError::ParseResponse { .. })));
    }
}
