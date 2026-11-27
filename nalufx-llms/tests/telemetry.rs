//! Provider calls leave spans an operator can read: one per attempt, with
//! the provider, attempt number, status and latency. No span field or
//! event ever holds the API key, the prompt or the reply.

use nalufx_llms::adapters::openai_compatible::{OpenAiCompatible, OPENAI};
use nalufx_llms::http::Http;
use nalufx_llms::provider::{ChatRequest, ClientConfig, LlmConfig, LlmProvider};
use std::fmt::Write as _;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::field::{Field, Visit};
use tracing::span::{Attributes, Id, Record};
use tracing::{Event, Subscriber};
use tracing_subscriber::layer::{Context, Layer, SubscriberExt};
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

const KEY: &str = "sk-telemetry-key-must-not-appear";
const PROMPT: &str = "secret investor context that must not appear";
const REPLY: &str = "private reply text that must not appear";

/// Every span and event, as `name field=value ...` lines.
#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<String>>>);

struct Line<'a>(&'a mut String);

impl Visit for Line<'_> {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let _ = write!(self.0, " {}={value:?}", field.name());
    }
}

impl<S: Subscriber> Layer<S> for Capture {
    fn on_new_span(&self, attrs: &Attributes<'_>, _: &Id, _: Context<'_, S>) {
        let mut line = format!("span {}", attrs.metadata().name());
        attrs.record(&mut Line(&mut line));
        self.0.lock().unwrap().push(line);
    }

    fn on_record(&self, _: &Id, values: &Record<'_>, _: Context<'_, S>) {
        let mut line = "record".to_string();
        values.record(&mut Line(&mut line));
        self.0.lock().unwrap().push(line);
    }

    fn on_event(&self, event: &Event<'_>, _: Context<'_, S>) {
        let mut line = format!("event {}", event.metadata().level());
        event.record(&mut Line(&mut line));
        self.0.lock().unwrap().push(line);
    }
}

#[tokio::test]
async fn each_attempt_has_a_span_and_nothing_private_is_recorded() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500).set_body_string(REPLY))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    let reply =
        serde_json::json!({ "choices": [{ "message": { "content": REPLY } }], "model": "m" });
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(reply))
        .mount(&server)
        .await;

    let capture = Capture::default();
    let _guard =
        tracing::subscriber::set_default(tracing_subscriber::registry().with(capture.clone()));
    let client = ClientConfig {
        max_attempts: 2,
        base_delay: Duration::from_millis(1),
        ..ClientConfig::default()
    };
    let provider = OpenAiCompatible::new(
        Http::new(client).unwrap(),
        OPENAI,
        format!("{}/v1", server.uri()),
        Some(KEY.into()),
        LlmConfig::new("m", 64).unwrap(),
    )
    .unwrap();
    provider.complete(&ChatRequest::user(Some("system"), PROMPT)).await.unwrap();

    let lines = capture.0.lock().unwrap().clone();
    let attempts: Vec<&String> =
        lines.iter().filter(|l| l.starts_with("span llm.attempt")).collect();
    assert_eq!(attempts.len(), 2, "{lines:#?}");
    assert!(attempts[0].contains("provider=\"openai\"") && attempts[0].contains("attempt=1"));
    assert!(attempts[1].contains("attempt=2"));
    let records: Vec<&String> = lines.iter().filter(|l| l.starts_with("record")).collect();
    assert!(records.iter().any(|l| l.contains("status=\"500\"")), "{lines:#?}");
    assert!(records.iter().any(|l| l.contains("status=\"200\"")), "{lines:#?}");
    assert!(records.iter().any(|l| l.contains("latency_ms=")), "{lines:#?}");
    assert!(lines.iter().any(|l| l.starts_with("event WARN") && l.contains("retrying")));
    for line in &lines {
        for private in [KEY, PROMPT, REPLY] {
            assert!(!line.contains(private), "{private:?} recorded in {line}");
        }
    }
}
