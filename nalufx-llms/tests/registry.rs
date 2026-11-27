//! Providers built by name from the environment, called against a local
//! server through the base-URL override. Environment variables are
//! process-wide, so these tests run one at a time behind a lock, in their
//! own test binary.

use nalufx_llms::error::LlmError;
use nalufx_llms::http::Http;
use nalufx_llms::provider::{ChatRequest, ClientConfig, LlmConfig};
use nalufx_llms::registry::{from_env, BASE_URL_VAR, PROVIDERS};
use serde_json::json;
use tokio::sync::Mutex;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

static ENV: Mutex<()> = Mutex::const_new(());

fn config() -> LlmConfig {
    LlmConfig::new("test-model", 64).unwrap()
}

fn http() -> Http {
    Http::new(ClientConfig::default()).unwrap()
}

/// Sets each variable for the duration of `f`, then removes it.
async fn with_env<F: std::future::Future<Output = ()>>(
    vars: &[(&str, &str)],
    f: impl FnOnce() -> F,
) {
    let _guard = ENV.lock().await;
    for (k, v) in vars {
        std::env::set_var(k, v);
    }
    f().await;
    for (k, _) in vars {
        std::env::remove_var(k);
    }
}

async fn server(route: &str, key_header: (&str, &str), reply: serde_json::Value) -> MockServer {
    let s = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(route))
        .and(header(key_header.0, key_header.1))
        .respond_with(ResponseTemplate::new(200).set_body_json(reply))
        .expect(1)
        .mount(&s)
        .await;
    s
}

#[tokio::test]
async fn anthropic_reads_its_key_and_the_base_url_override() {
    let reply = json!({ "content": [{ "type": "text", "text": "hi" }], "model": "m", "usage": { "input_tokens": 1, "output_tokens": 1 } });
    let s = server("/v1/messages", ("x-api-key", "env-key"), reply).await;
    with_env(&[("ANTHROPIC_API_KEY", "env-key"), (BASE_URL_VAR, &s.uri())], || async {
        let p = from_env("anthropic", config(), http()).unwrap();
        assert_eq!((p.name(), p.model()), ("anthropic", "test-model"));
        assert_eq!(p.complete(&ChatRequest::user(None, "x")).await.unwrap().text, "hi");
    })
    .await;
}

#[tokio::test]
async fn gemini_reads_its_key_and_the_base_url_override() {
    let reply = json!({ "candidates": [{ "content": { "parts": [{ "text": "hi" }] } }] });
    let s =
        server("/v1beta/models/test-model:generateContent", ("x-goog-api-key", "env-key"), reply)
            .await;
    with_env(&[("GEMINI_API_KEY", "env-key"), (BASE_URL_VAR, &s.uri())], || async {
        let p = from_env("gemini", config(), http()).unwrap();
        assert_eq!(p.complete(&ChatRequest::user(None, "x")).await.unwrap().text, "hi");
    })
    .await;
}

#[tokio::test]
async fn openai_compatible_providers_read_their_keys() {
    let reply = json!({ "choices": [{ "message": { "content": "hi" } }] });
    for (name, var) in
        [("openai", "OPENAI_API_KEY"), ("mistral", "MISTRAL_API_KEY"), ("agy", "AGY_API_KEY")]
    {
        let s = server("/v1/chat/completions", ("authorization", "Bearer env-key"), reply.clone())
            .await;
        let url = format!("{}/v1", s.uri());
        with_env(&[(var, "env-key"), (BASE_URL_VAR, &url)], || async {
            let p = from_env(name, config(), http()).unwrap();
            assert_eq!(p.name(), name);
            assert_eq!(
                p.complete(&ChatRequest::user(None, "x")).await.unwrap().text,
                "hi",
                "{name}"
            );
        })
        .await;
    }
}

#[tokio::test]
async fn native_providers_keep_their_public_endpoint_without_an_override() {
    with_env(&[("ANTHROPIC_API_KEY", "k"), ("GEMINI_API_KEY", "k")], || async {
        for name in ["anthropic", "gemini"] {
            assert_eq!(from_env(name, config(), http()).unwrap().name(), name);
        }
    })
    .await;
}

#[tokio::test]
async fn agy_without_a_base_url_is_a_configuration_error() {
    with_env(&[("AGY_API_KEY", "k")], || async {
        let err = from_env("agy", config(), http()).err().expect("an error");
        assert!(matches!(&err, LlmError::Config(m) if m.contains(BASE_URL_VAR)), "{err}");
    })
    .await;
}

#[test]
fn every_listed_provider_is_known() {
    assert_eq!(PROVIDERS.len(), 7);
    assert!(PROVIDERS.contains(&"openai_compatible"));
}
