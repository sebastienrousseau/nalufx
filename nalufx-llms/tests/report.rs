//! The M3 exit gate: a report generated through each adapter kind passes
//! schema validation, and one that does not match is an error, never text.

use nalufx_llms::adapters::anthropic::Anthropic;
use nalufx_llms::adapters::gemini::Gemini;
use nalufx_llms::adapters::openai_compatible::{OpenAiCompatible, OLLAMA};
use nalufx_llms::error::LlmError;
use nalufx_llms::http::Http;
use nalufx_llms::provider::{ClientConfig, LlmConfig, LlmProvider};
use nalufx_llms::report::{generate, ReportInput, SCHEMA};
use serde_json::{json, Value};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const REPORT: &str = r#"{"summary":"Minimum variance puts most of the money in bonds.","rationale":["Bonds have a fifth of the volatility of stocks.","Their correlation is low."],"risks":["Rising rates hurt bonds."],"disclaimer":"For research and analysis only; not investment advice."}"#;

fn input() -> ReportInput {
    ReportInput {
        portfolio: "Treasury reserve".into(),
        facts: json!({ "method": "min_variance", "assets": ["BONDS", "STOCKS"], "weights": [0.82, 0.18] }),
        context: "Preserve capital; spend in 6 months.".into(),
    }
}

fn config() -> LlmConfig {
    LlmConfig::new("test-model", 1024).unwrap()
}

fn http() -> Http {
    Http::new(ClientConfig::default()).unwrap()
}

/// A provider and the path its server answers, replying with `text` in
/// the provider's own envelope.
/// A provider for a mock server's URL.
type Build = fn(&str) -> Box<dyn LlmProvider>;

fn kinds(text: &str) -> Vec<(&'static str, Value, Build)> {
    vec![
        (
            "/v1/messages",
            json!({ "content": [{ "type": "text", "text": text }], "model": "claude-test", "usage": { "input_tokens": 900, "output_tokens": 80 } }),
            |url| Box::new(Anthropic::new(http(), "k", config()).unwrap().with_base_url(url)),
        ),
        (
            "/v1beta/models/test-model:generateContent",
            json!({ "candidates": [{ "content": { "parts": [{ "text": text }] } }], "modelVersion": "gemini-test" }),
            |url| Box::new(Gemini::new(http(), "k", config()).unwrap().with_base_url(url)),
        ),
        (
            "/v1/chat/completions",
            json!({ "choices": [{ "message": { "content": text } }], "model": "llama-test" }),
            |url| {
                Box::new(
                    OpenAiCompatible::new(http(), OLLAMA, format!("{url}/v1"), None, config())
                        .unwrap(),
                )
            },
        ),
    ]
}

async fn provider_replying(
    path_: &str,
    reply: Value,
    build: Build,
) -> (MockServer, Box<dyn LlmProvider>) {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(path_))
        .respond_with(ResponseTemplate::new(200).set_body_json(reply))
        .mount(&server)
        .await;
    let provider = build(&server.uri());
    (server, provider)
}

#[tokio::test]
async fn every_adapter_kind_produces_a_valid_report() {
    for (route, reply, build) in kinds(REPORT) {
        let (server, provider) = provider_replying(route, reply, build).await;
        let report =
            generate(provider.as_ref(), &input()).await.unwrap_or_else(|e| panic!("{route}: {e}"));
        assert_eq!(report.report.rationale.len(), 2);
        assert_eq!((report.prompt.name.as_str(), report.prompt.version), ("allocation_report", 1));
        assert_eq!(report.provider, provider.name());

        // The request carried the figures, the schema and the context.
        let sent =
            String::from_utf8(server.received_requests().await.unwrap()[0].body.clone()).unwrap();
        for expected in [
            "0.82",
            "min_variance",
            "Preserve capital",
            "additionalProperties",
            "not investment advice",
        ] {
            assert!(sent.contains(expected), "{route}: the request lacks {expected:?}");
        }
    }
}

#[tokio::test]
async fn a_reply_off_the_schema_is_an_error_for_every_adapter_kind() {
    let off_schema = r#"{"summary":"Buy bonds.","weights":[0.9,0.1]}"#;
    for (route, reply, build) in kinds(off_schema) {
        let (_server, provider) = provider_replying(route, reply, build).await;
        let err = generate(provider.as_ref(), &input()).await.unwrap_err();
        assert!(matches!(err, LlmError::Schema { schema: SCHEMA, .. }), "{route}: {err}");
    }
}
