//! Logs and spans.
//!
//! Every request runs in a `request` span with its method, route,
//! request id, the id of the key that made it, its status and latency.
//! The handlers add child spans (`forecast`, `allocate`, `report`), and
//! nalufx-llms one per provider attempt (`llm.attempt`).
//!
//! Logs are written by rlg (RustLogs): JSON lines by default, or logfmt
//! with `NALUFX_LOG_FORMAT=text`, at `RUST_LOG`'s level (default
//! `info`). Every line carries the fields of the spans it sits in, so
//! each one has the request's id, key id and route, plus `trace_id` and
//! `span_id`. rlg writes to journald when the host has it, and to
//! standard output otherwise (or with `RLG_FALLBACK_STDOUT=1`). Before a
//! line or a span leaves the process, rlg-redact scrubs card numbers,
//! JWTs, bearer tokens, emails, IPv4 addresses, AWS key ids and
//! language model provider keys from it, a second line of defence.
//!
//! With `OTEL_EXPORTER_OTLP_ENDPOINT` set, closed spans also go to that
//! OpenTelemetry collector as OTLP/HTTP traces (rlg-otlp), with the
//! service name `nalufx-api` unless `OTEL_SERVICE_NAME` says otherwise.
//!
//! What never goes into a span or a log: an API key (only its
//! [`key_id`]), a prompt, the investor's words, or a provider's reply.

use crate::metrics::route;
use crate::problem::REQUEST_ID;
use axum::http::{Request, Response};
use rlg::tracing::RlgLayer;
use rlg_otlp::SpanExporter;
use rlg_redact::Redactor;
use std::time::Duration;
use tracing::Span;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Language model provider keys (OpenAI-style `sk-`, Anthropic's
/// `sk-ant-`), which rlg-redact's defaults do not cover.
const PROVIDER_KEY: &str = r"\bsk-(?:ant-)?[A-Za-z0-9_-]{20,}\b";

/// How log lines are written.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LogFormat {
    /// One JSON object per line, for a log pipeline.
    #[default]
    Json,
    /// Human-readable lines.
    Text,
}

impl std::str::FromStr for LogFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(Self::Json),
            "text" => Ok(Self::Text),
            other => Err(format!("NALUFX_LOG_FORMAT must be json or text, not `{other}`")),
        }
    }
}

/// A key's id in spans and the audit log: the first 8 hex characters of
/// its SHA-256, the hash `NALUFX_API_KEY_SHA256` already lists. It names
/// the key to the operator without revealing it.
#[must_use]
pub fn key_id(hash: &[u8; 32]) -> String {
    hex::encode(&hash[..4])
}

/// The span a request runs in. The key id, status and latency are filled
/// in as they become known.
pub fn request_span<B>(request: &Request<B>) -> Span {
    let id = request.headers().get(REQUEST_ID).and_then(|v| v.to_str().ok()).unwrap_or("");
    tracing::info_span!(
        "request",
        method = %request.method(),
        route = route(request.uri().path()),
        request_id = id,
        key_id = tracing::field::Empty,
        status = tracing::field::Empty,
        latency_ms = tracing::field::Empty,
        otel.kind = "server",
        otel.status_code = tracing::field::Empty,
    )
}

/// Records the status and latency on the request span, and logs one line
/// for the request.
pub fn on_response<B>(response: &Response<B>, latency: Duration, span: &Span) {
    let status = response.status().as_u16();
    let _ = span.record("status", status);
    let _ = span.record("latency_ms", u64::try_from(latency.as_millis()).unwrap_or(u64::MAX));
    let _ = span.record("otel.status_code", if status >= 500 { "error" } else { "ok" });
    tracing::info!(parent: span, status, "answered");
}

/// What [`init`] started: flush it before the process exits.
#[derive(Debug)]
pub struct Telemetry {
    spans: Option<SpanExporter>,
}

impl Telemetry {
    /// Sends the spans still queued (waiting up to five seconds) and
    /// writes the log lines still buffered.
    pub fn shutdown(self) {
        if let Some(spans) = &self.spans {
            if !spans.flush(Duration::from_secs(5)) {
                tracing::warn!("some spans were not exported before shutdown");
            }
        }
        rlg::engine::ENGINE.shutdown();
    }
}

/// The rlg layer for `format`, scrubbing what it writes, and exporting
/// spans to `spans` if there is an exporter.
///
/// # Errors
///
/// If the provider-key pattern does not compile, which it does.
pub fn layer(format: LogFormat, spans: Option<&SpanExporter>) -> Result<RlgLayer, String> {
    let scrub = Redactor::with_defaults().with_pattern(PROVIDER_KEY).map_err(|e| e.to_string())?;
    let format = match format {
        LogFormat::Json => rlg::LogFormat::JSON,
        LogFormat::Text => rlg::LogFormat::Logfmt,
    };
    let mut layer = RlgLayer::new().with_format(format).with_scrub(scrub);
    if let Some(exporter) = spans {
        layer = layer.with_span_sink(exporter.clone());
    }
    Ok(layer)
}

/// The span exporter the OpenTelemetry variables ask for, if any. The
/// service is `nalufx-api` unless `OTEL_SERVICE_NAME` names another.
///
/// # Errors
///
/// If the endpoint is unusable (rlg-otlp takes plain `http://` only:
/// send to a local collector, which handles TLS).
pub fn span_exporter(get: impl Fn(&str) -> Option<String>) -> Result<Option<SpanExporter>, String> {
    let lookup = |name: &str| {
        get(name).or_else(|| (name == "OTEL_SERVICE_NAME").then(|| "nalufx-api".to_string()))
    };
    rlg_otlp::SpanExporterBuilder::from_lookup(lookup)
        .map(|b| b.build().map_err(|e| format!("OpenTelemetry export: {e}")))
        .transpose()
}

/// Installs the global subscriber: rlg lines in `format` at `RUST_LOG`'s
/// level, and span export if the OpenTelemetry variables ask for it.
///
/// # Errors
///
/// If the exporter cannot be built, or a subscriber is already
/// installed.
pub fn init(format: LogFormat) -> Result<Telemetry, String> {
    let spans = span_exporter(|name| std::env::var(name).ok())?;
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(filter)
        .with(layer(format, spans.as_ref())?)
        .try_init()
        .map_err(|e| format!("logging could not start: {e}"))?;
    if let Some(exporter) = &spans {
        tracing::info!(endpoint = exporter.endpoint(), "exporting spans");
    }
    Ok(Telemetry { spans })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_id_is_a_short_hash_prefix() {
        let hash = crate::auth::hash_key("test");
        assert_eq!(key_id(&hash), "9f86d081");
    }

    #[test]
    fn the_log_format_is_json_or_text() {
        assert_eq!("json".parse::<LogFormat>(), Ok(LogFormat::Json));
        assert_eq!("text".parse::<LogFormat>(), Ok(LogFormat::Text));
        assert!("xml".parse::<LogFormat>().is_err());
        assert_eq!(LogFormat::default(), LogFormat::Json);
    }
}
