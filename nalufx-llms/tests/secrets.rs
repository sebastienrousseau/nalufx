//! API keys never reach a debug print, a log line or an error message.

use nalufx_llms::adapters::anthropic::Anthropic;
use nalufx_llms::adapters::gemini::Gemini;
use nalufx_llms::adapters::openai_compatible::{OpenAiCompatible, OPENAI};
use nalufx_llms::http::Http;
use nalufx_llms::provider::{ClientConfig, LlmConfig};

const KEY: &str = "sk-test-0123456789-must-never-be-printed";

fn parts() -> (Http, LlmConfig) {
    (Http::new(ClientConfig::default()).unwrap(), LlmConfig::new("m", 64).unwrap())
}

#[test]
fn no_adapter_prints_its_key() {
    let (http, config) = parts();
    let printed = [
        format!("{:?}", Anthropic::new(http.clone(), KEY, config.clone()).unwrap()),
        format!("{:?}", Gemini::new(http.clone(), KEY, config.clone()).unwrap()),
        format!(
            "{:?}",
            OpenAiCompatible::new(
                http,
                OPENAI,
                "https://example.test/v1",
                Some(KEY.into()),
                config
            )
            .unwrap()
        ),
    ];
    for p in printed {
        assert!(!p.contains(KEY), "a key leaked: {p}");
        assert!(p.contains("[REDACTED]"), "the key's place is not marked: {p}");
    }
}
