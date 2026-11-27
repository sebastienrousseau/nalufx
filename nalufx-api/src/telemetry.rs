//! Logs and spans.
//!
//! Every request runs in a `request` span with its method, route,
//! request id, the id of the key that made it, its status and latency.
//! The handlers add child spans (`forecast`, `allocate`, `report`), and
//! nalufx-llms one per provider attempt (`llm.attempt`). Logs are JSON
//! lines by default, or text with `NALUFX_LOG_FORMAT=text`; the level
//! comes from `RUST_LOG` (default `info`).
//!
//! What never goes into a span or a log: an API key (only its
//! [`key_id`]), a prompt, the investor's words, or a provider's reply.

use crate::metrics::route;
use crate::problem::REQUEST_ID;
use axum::http::{Request, Response};
use std::time::Duration;
use tracing::Span;

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
    )
}

/// Records the status and latency on the request span, and logs one line
/// for the request.
pub fn on_response<B>(response: &Response<B>, latency: Duration, span: &Span) {
    let status = response.status().as_u16();
    let _ = span.record("status", status);
    let _ = span.record("latency_ms", u64::try_from(latency.as_millis()).unwrap_or(u64::MAX));
    tracing::info!(parent: span, status, "answered");
}

/// Installs the global subscriber: JSON or text lines at `RUST_LOG`'s
/// level, with each line's span fields.
///
/// # Errors
///
/// If a subscriber is already installed.
pub fn init(format: LogFormat) -> Result<(), String> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    let builder = tracing_subscriber::fmt().with_env_filter(filter);
    let result = match format {
        LogFormat::Json => builder.json().with_current_span(true).with_span_list(false).try_init(),
        LogFormat::Text => builder.try_init(),
    };
    result.map_err(|e| format!("logging could not start: {e}"))
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
