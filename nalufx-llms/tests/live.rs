//! Live calls to the real providers. Ignored by default: no test in the
//! normal run reaches a provider. The nightly `llm-live` workflow runs
//! them with `--ignored`, with keys from repository secrets.
//!
//! Each provider runs when its key (`ANTHROPIC_API_KEY`,
//! `GEMINI_API_KEY`, `OPENAI_API_KEY`) and model (`NALUFX_LIVE_ANTHROPIC_MODEL`,
//! `NALUFX_LIVE_GEMINI_MODEL`, `NALUFX_LIVE_OPENAI_MODEL`) are set, and is
//! skipped with a message otherwise.

use nalufx_llms::http::Http;
use nalufx_llms::provider::{ClientConfig, LlmConfig};
use nalufx_llms::registry::from_env;
use nalufx_llms::report::{generate, ReportInput};
use serde_json::json;

async fn live_report(provider: &str, key_var: &str, model_var: &str) {
    let (Ok(_), Ok(model)) = (std::env::var(key_var), std::env::var(model_var)) else {
        eprintln!("skipped {provider}: set {key_var} and {model_var}");
        return;
    };
    let config = LlmConfig::new(model, 1024).expect("a valid config");
    let p = from_env(provider, config, Http::new(ClientConfig::default()).unwrap())
        .expect("a provider");
    let input = ReportInput {
        portfolio: "Live check".into(),
        facts: json!({ "method": "min_variance", "assets": ["BONDS", "STOCKS"], "weights": [0.8, 0.2] }),
        context: "A nightly check that the provider still answers in schema.".into(),
    };
    let report = generate(p.as_ref(), &input).await.unwrap_or_else(|e| panic!("{provider}: {e}"));
    eprintln!(
        "{provider} ({}): {} tokens in, {} out",
        report.model, report.usage.input_tokens, report.usage.output_tokens
    );
}

#[tokio::test]
#[ignore = "calls Anthropic; run by the nightly llm-live workflow"]
async fn anthropic_writes_a_valid_report() {
    live_report("anthropic", "ANTHROPIC_API_KEY", "NALUFX_LIVE_ANTHROPIC_MODEL").await;
}

#[tokio::test]
#[ignore = "calls Gemini; run by the nightly llm-live workflow"]
async fn gemini_writes_a_valid_report() {
    live_report("gemini", "GEMINI_API_KEY", "NALUFX_LIVE_GEMINI_MODEL").await;
}

#[tokio::test]
#[ignore = "calls OpenAI; run by the nightly llm-live workflow"]
async fn openai_writes_a_valid_report() {
    live_report("openai", "OPENAI_API_KEY", "NALUFX_LIVE_OPENAI_MODEL").await;
}
