//! Tests for the `/predict` endpoint and the OpenAI adapter.
//!
//! The `/predict` cases drive a real server on an ephemeral port rather
//! than a mock handler. The endpoint makes no network call: it forecasts
//! with AutoETS. The adapter cases use a local mock provider.

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;
    use nalufx::{
        api::server::serve_on,
        models::cash_flow_dm::{CashFlowResponse, ProblemDetails},
    };
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

    /// Synthetic history long enough for AutoETS to fit.
    fn history(n: usize) -> Vec<f64> {
        (0..n).map(|i| 1000.0 + 25.0 * (i as f64 * 0.7).sin() + i as f64).collect()
    }

    /// Three assets' returns over `n` days: a calm one, a volatile one and
    /// one between.
    fn returns(n: usize) -> Vec<Vec<f64>> {
        (0..n)
            .map(|i| {
                let x = i as f64;
                vec![0.001 * (x * 0.7).sin(), 0.02 * (x * 1.3).cos(), 0.008 * (x * 0.4).sin()]
            })
            .collect()
    }

    /// A request body with the three assets.
    fn body(historical_data: Vec<f64>, returns: Vec<Vec<f64>>) -> serde_json::Value {
        json!({ "historical_data": historical_data, "assets": ["BONDS", "STOCKS", "CREDIT"], "returns": returns })
    }

    async fn post_predict(body: serde_json::Value) -> reqwest::Response {
        let base = spawn_server().await;
        Client::new()
            .post(format!("{base}/predict"))
            .json(&body)
            .send()
            .await
            .expect("should reach the server")
    }

    async fn problem(response: reqwest::Response) -> ProblemDetails {
        assert_eq!(response.headers()["content-type"], "application/problem+json");
        response.json().await.expect("should be a problem details body")
    }

    #[tokio::test]
    async fn a_valid_request_gets_a_forecast_with_its_interval_and_an_allocation() {
        let response = post_predict(body(history(60), returns(60))).await;
        assert_eq!(response.status(), 200);
        let body: CashFlowResponse = response.json().await.expect("should be a CashFlowResponse");
        assert_eq!(body.forecast.level, 0.95);
        assert_eq!(body.forecast.point.len(), 6);
        for day in 0..6 {
            let (lo, p, hi) =
                (body.forecast.lower[day], body.forecast.point[day], body.forecast.upper[day]);
            assert!(lo <= p && p <= hi, "day {day}: {lo} <= {p} <= {hi}");
        }
        let a = &body.allocation;
        assert_eq!(a.method, "min_variance");
        assert_eq!(a.assets, vec!["BONDS", "STOCKS", "CREDIT"]);
        assert!(a.weights.iter().all(|w| *w >= 0.0));
        assert!((a.weights.iter().sum::<f64>() - 1.0).abs() < 1e-9);
        // The calmest asset gets the most under minimum variance.
        assert!(a.weights[0] > a.weights[1], "{:?}", a.weights);
        assert_eq!(a.params["estimator"], "ledoit_wolf");
        // The old handler replaced a short forecast with six zeros.
        assert_ne!(body.forecast.point, vec![0.0; 6]);
    }

    #[tokio::test]
    async fn the_same_request_gets_the_same_bytes() {
        let body = body(history(60), returns(60));
        let first = post_predict(body.clone()).await.bytes().await.expect("body");
        let second = post_predict(body).await.bytes().await.expect("body");
        assert_eq!(first, second);
    }

    #[tokio::test]
    async fn empty_historical_data_is_a_bad_request_problem() {
        let response = post_predict(body(vec![], returns(60))).await;
        assert_eq!(response.status(), 400);
        let body = problem(response).await;
        assert_eq!(body.status, 400);
        assert_eq!(body.title, "Bad Request");
        assert_eq!(body.detail, "Input slices cannot be empty");
    }

    #[tokio::test]
    async fn bounds_no_weights_can_meet_are_unprocessable() {
        let mut request = body(history(60), returns(60));
        request["constraints"] = json!({ "max_weight": 0.3 });
        let response = post_predict(request).await;
        assert_eq!(response.status(), 422);
        assert!(problem(response).await.detail.starts_with("The constraints cannot be satisfied"));
    }

    #[tokio::test]
    async fn every_method_answers_by_name() {
        for method in ["equal_weight", "min_variance", "mean_variance", "risk_parity", "hrp"] {
            let mut request = body(history(60), returns(60));
            request["method"] = json!(method);
            let response = post_predict(request).await;
            assert_eq!(response.status(), 200, "{method}");
            let body: CashFlowResponse = response.json().await.expect("a CashFlowResponse");
            assert_eq!(body.allocation.method, method);
        }
    }

    #[tokio::test]
    async fn a_seasonal_request_is_forecast_with_mstl() {
        let cycle = [0.0, 40.0, 10.0, -20.0, 60.0];
        let history: Vec<f64> = (0..60).map(|i| 1000.0 + i as f64 + cycle[i % 5]).collect();
        let mut request = body(history, returns(60));
        request["seasonal_periods"] = json!([5]);
        let response = post_predict(request).await;
        assert_eq!(response.status(), 200);
        let body: CashFlowResponse = response.json().await.expect("a CashFlowResponse");
        // Day 61 is position 0 of the cycle; position 4 (day 65) is the peak.
        let p = &body.forecast.point;
        assert!(p[4] > p[3] && p[4] > p[0], "{p:?}");
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
        assert!(problem(response).await.detail.starts_with("Invalid request body"));
    }

    #[tokio::test]
    async fn a_body_missing_a_series_is_rejected() {
        let response = post_predict(json!({ "historical_data": [1.0, 2.0, 3.0] })).await;
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
        assert_eq!(problem(response).await.status, 404);
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
