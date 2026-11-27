//! Contract tests: each adapter against a local server playing its
//! provider, from recorded replies. Every adapter is held to the same
//! cases: the request it sends, a success, a 429 with `Retry-After`, a
//! 500, a 400, a timeout, a malformed body and a reply with no text.

use nalufx_llms::adapters::anthropic::Anthropic;
use nalufx_llms::adapters::gemini::Gemini;
use nalufx_llms::adapters::openai_compatible::{OpenAiCompatible, OPENAI};
use nalufx_llms::error::LlmError;
use nalufx_llms::http::Http;
use nalufx_llms::provider::{ChatRequest, ClientConfig, LlmConfig, LlmProvider, TokenUsage};
use serde_json::json;
use std::time::{Duration, Instant};
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// How one adapter is called and what its provider returns.
struct Adapter {
    path: &'static str,
    success: &'static str,
    /// A 200 reply with no text in it.
    empty: serde_json::Value,
    build: fn(Http, &str) -> Box<dyn LlmProvider>,
}

fn config() -> LlmConfig {
    let mut c = LlmConfig::new("test-model", 256).expect("a valid config");
    c.temperature = Some(0.2);
    c
}

fn anthropic() -> Adapter {
    Adapter {
        path: "/v1/messages",
        success: include_str!("fixtures/anthropic_success.json"),
        empty: json!({ "content": [], "model": "m", "usage": { "input_tokens": 1, "output_tokens": 0 } }),
        build: |http, url| {
            Box::new(Anthropic::new(http, "test-key", config()).unwrap().with_base_url(url))
        },
    }
}

fn gemini() -> Adapter {
    Adapter {
        path: "/v1beta/models/test-model:generateContent",
        success: include_str!("fixtures/gemini_success.json"),
        empty: json!({ "candidates": [] }),
        build: |http, url| {
            Box::new(Gemini::new(http, "test-key", config()).unwrap().with_base_url(url))
        },
    }
}

fn openai() -> Adapter {
    Adapter {
        path: "/v1/chat/completions",
        success: include_str!("fixtures/openai_success.json"),
        empty: json!({ "choices": [] }),
        build: |http, url| {
            Box::new(
                OpenAiCompatible::new(
                    http,
                    OPENAI,
                    format!("{url}/v1"),
                    Some("test-key".into()),
                    config(),
                )
                .unwrap(),
            )
        },
    }
}

/// A client that retries quickly, so the tests run fast.
fn http(max_attempts: u32, request_timeout: Duration) -> Http {
    Http::new(ClientConfig {
        connect_timeout: Duration::from_secs(1),
        request_timeout,
        max_attempts,
        base_delay: Duration::from_millis(1),
        max_delay: Duration::from_secs(5),
    })
    .expect("a client")
}

fn request() -> ChatRequest {
    ChatRequest::user(Some("Be brief."), "Describe the allocation.")
}

async fn serve(adapter: &Adapter, response: ResponseTemplate) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST")).and(path(adapter.path)).respond_with(response).mount(&server).await;
    server
}

async fn succeeds(adapter: Adapter, provider: &str, model: &str) {
    let server = serve(&adapter, ResponseTemplate::new(200).set_body_string(adapter.success)).await;
    let p = (adapter.build)(http(3, Duration::from_secs(5)), &server.uri());
    let r = p.complete(&request()).await.expect("a success");
    assert_eq!(r.text, "The allocation leans on bonds.");
    assert_eq!(r.usage, TokenUsage { input_tokens: 412, output_tokens: 37 });
    assert_eq!((r.provider.as_str(), r.model.as_str()), (provider, model));
}

async fn honours_retry_after(adapter: Adapter) {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(adapter.path))
        .respond_with(ResponseTemplate::new(429).insert_header("retry-after", "1"))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(adapter.path))
        .respond_with(ResponseTemplate::new(200).set_body_string(adapter.success))
        .mount(&server)
        .await;
    let p = (adapter.build)(http(3, Duration::from_secs(5)), &server.uri());
    let start = Instant::now();
    assert!(p.complete(&request()).await.is_ok());
    assert!(start.elapsed() >= Duration::from_secs(1), "waited only {:?}", start.elapsed());
    assert_eq!(server.received_requests().await.unwrap().len(), 2);
}

async fn retries_a_server_error_three_times(adapter: Adapter) {
    let server = serve(&adapter, ResponseTemplate::new(500).set_body_string("overloaded")).await;
    let p = (adapter.build)(http(3, Duration::from_secs(5)), &server.uri());
    let err = p.complete(&request()).await.unwrap_err();
    assert!(
        matches!(&err, LlmError::Status { status: 500, body, .. } if body == "overloaded"),
        "{err}"
    );
    assert_eq!(server.received_requests().await.unwrap().len(), 3);
}

async fn does_not_retry_a_client_error(adapter: Adapter) {
    let server = serve(&adapter, ResponseTemplate::new(400).set_body_string("bad model")).await;
    let p = (adapter.build)(http(3, Duration::from_secs(5)), &server.uri());
    assert!(matches!(p.complete(&request()).await, Err(LlmError::Status { status: 400, .. })));
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

async fn times_out(adapter: Adapter) {
    let slow = ResponseTemplate::new(200)
        .set_body_string(adapter.success)
        .set_delay(Duration::from_secs(3));
    let server = serve(&adapter, slow).await;
    let p = (adapter.build)(http(2, Duration::from_millis(200)), &server.uri());
    let err = p.complete(&request()).await.unwrap_err();
    assert!(matches!(err, LlmError::Transport { timeout: true, .. }), "{err}");
    assert_eq!(server.received_requests().await.unwrap().len(), 2);
}

async fn rejects_a_malformed_body(adapter: Adapter) {
    let server = serve(&adapter, ResponseTemplate::new(200).set_body_string("{ not json")).await;
    let p = (adapter.build)(http(3, Duration::from_secs(5)), &server.uri());
    assert!(matches!(p.complete(&request()).await, Err(LlmError::ParseResponse { .. })));
    assert_eq!(
        server.received_requests().await.unwrap().len(),
        1,
        "a malformed body is not retried"
    );
}

async fn rejects_a_reply_with_no_text(adapter: Adapter) {
    let server = serve(&adapter, ResponseTemplate::new(200).set_body_json(&adapter.empty)).await;
    let p = (adapter.build)(http(3, Duration::from_secs(5)), &server.uri());
    assert!(matches!(p.complete(&request()).await, Err(LlmError::ParseResponse { .. })));
}

/// The same eight cases for every adapter.
macro_rules! contract {
    ($module:ident, $adapter:expr, $provider:expr, $model:expr) => {
        mod $module {
            use super::*;

            #[tokio::test]
            async fn a_success_is_parsed() {
                succeeds($adapter, $provider, $model).await;
            }

            #[tokio::test]
            async fn a_429_waits_for_retry_after() {
                honours_retry_after($adapter).await;
            }

            #[tokio::test]
            async fn a_500_is_retried_three_times() {
                retries_a_server_error_three_times($adapter).await;
            }

            #[tokio::test]
            async fn a_400_is_not_retried() {
                does_not_retry_a_client_error($adapter).await;
            }

            #[tokio::test]
            async fn a_slow_reply_times_out() {
                times_out($adapter).await;
            }

            #[tokio::test]
            async fn a_malformed_body_is_a_parse_error() {
                rejects_a_malformed_body($adapter).await;
            }

            #[tokio::test]
            async fn a_reply_with_no_text_is_a_parse_error() {
                rejects_a_reply_with_no_text($adapter).await;
            }
        }
    };
}

contract!(anthropic_contract, anthropic(), "anthropic", "claude-sonnet-5-5");
contract!(gemini_contract, gemini(), "gemini", "gemini-3.0-pro");
contract!(openai_contract, openai(), "openai", "gpt-5.1");

/// The requests each adapter sends: the provider's own headers, and the
/// body in the provider's own shape.
mod request_shape {
    use super::*;

    async fn sends(
        adapter: Adapter,
        headers: &[(&str, &str)],
        body: serde_json::Value,
        json_mode: serde_json::Value,
    ) {
        let server = MockServer::start().await;
        let mut mock =
            Mock::given(method("POST")).and(path(adapter.path)).and(body_partial_json(body));
        for (name, value) in headers {
            mock = mock.and(header(*name, *value));
        }
        mock.respond_with(ResponseTemplate::new(200).set_body_string(adapter.success))
            .expect(1)
            .mount(&server)
            .await;
        let p = (adapter.build)(http(1, Duration::from_secs(5)), &server.uri());
        p.complete(&request()).await.expect("the request matched the provider's shape");

        let json_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(adapter.path))
            .and(body_partial_json(json_mode))
            .respond_with(ResponseTemplate::new(200).set_body_string(adapter.success))
            .expect(1)
            .mount(&json_server)
            .await;
        let p = (adapter.build)(http(1, Duration::from_secs(5)), &json_server.uri());
        let mut json_request = request();
        json_request.json = true;
        p.complete(&json_request).await.expect("the JSON request matched the provider's shape");
    }

    #[tokio::test]
    async fn anthropic_uses_its_key_header_version_and_system_field() {
        let body = json!({
            "model": "test-model", "max_tokens": 256, "temperature": 0.2,
            "system": "Be brief.",
            "messages": [{ "role": "user", "content": "Describe the allocation." }]
        });
        // Anthropic has no JSON mode: the request is the same.
        let headers = [("x-api-key", "test-key"), ("anthropic-version", "2023-06-01")];
        sends(anthropic(), &headers, body, json!({ "model": "test-model" })).await;
    }

    #[tokio::test]
    async fn gemini_uses_its_key_header_and_contents_shape() {
        let body = json!({
            "contents": [{ "role": "user", "parts": [{ "text": "Describe the allocation." }] }],
            "systemInstruction": { "parts": [{ "text": "Be brief." }] },
            "generationConfig": { "maxOutputTokens": 256, "temperature": 0.2 }
        });
        let json_mode = json!({ "generationConfig": { "responseMimeType": "application/json" } });
        sends(gemini(), &[("x-goog-api-key", "test-key")], body, json_mode).await;
    }

    #[tokio::test]
    async fn openai_uses_a_bearer_token_and_a_system_message() {
        let body = json!({
            "model": "test-model", "max_tokens": 256, "temperature": 0.2,
            "messages": [
                { "role": "system", "content": "Be brief." },
                { "role": "user", "content": "Describe the allocation." }
            ]
        });
        let json_mode = json!({ "response_format": { "type": "json_object" } });
        sends(openai(), &[("authorization", "Bearer test-key")], body, json_mode).await;
    }
}
