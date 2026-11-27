// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Errors produced by the LLM providers.
//!
//! They are transport-agnostic: mapping one to an HTTP status is the
//! caller's job, and lives in `nalufx-core`. Every variant is `Send` and
//! `Sync`, so an error can cross an `await` or a thread.

use std::time::Duration;
use thiserror::Error;

/// Why a completion could not be produced.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum LlmError {
    /// The caller's input cannot be sent: too long, for instance.
    #[error("unusable input: {0}")]
    Input(String),

    /// The provider's API key is not set.
    #[error("missing API key for {provider}: set {variable}")]
    MissingApiKey {
        /// The provider.
        provider: &'static str,
        /// The environment variable that holds it.
        variable: &'static str,
    },

    /// The request never got an HTTP response: a connection failure or a
    /// timeout.
    #[error("request to {provider} failed: {message}")]
    Transport {
        /// The provider.
        provider: &'static str,
        /// What went wrong.
        message: String,
        /// Whether it was a timeout.
        timeout: bool,
    },

    /// The provider answered with an error status.
    #[error("{provider} returned HTTP {status}: {body}")]
    Status {
        /// The provider.
        provider: &'static str,
        /// The HTTP status code.
        status: u16,
        /// How long the provider asked the caller to wait (`Retry-After`).
        retry_after: Option<Duration>,
        /// The start of the response body, for diagnosis.
        body: String,
    },

    /// The provider's response is not the shape its API documents.
    #[error("could not parse the {provider} response: {message}")]
    ParseResponse {
        /// The provider.
        provider: &'static str,
        /// What did not parse.
        message: String,
    },

    /// A structured reply did not match its schema.
    #[error("the {provider} reply does not match the {schema} schema: {message}")]
    Schema {
        /// The provider.
        provider: &'static str,
        /// The schema's name.
        schema: &'static str,
        /// What did not match.
        message: String,
    },

    /// The configuration is not usable (an empty model name, a bad URL).
    #[error("invalid LLM configuration: {0}")]
    Config(String),
}

impl LlmError {
    /// The provider the error came from; `"config"` for a configuration
    /// error and `"input"` for unusable input.
    #[must_use]
    pub const fn provider(&self) -> &'static str {
        match self {
            Self::MissingApiKey { provider, .. }
            | Self::Transport { provider, .. }
            | Self::Status { provider, .. }
            | Self::ParseResponse { provider, .. }
            | Self::Schema { provider, .. } => provider,
            Self::Config(_) => "config",
            Self::Input(_) => "input",
        }
    }

    /// Whether trying again may succeed: a transport failure or timeout,
    /// `429 Too Many Requests`, or a `5xx`. A `4xx`, a parse or schema
    /// failure, or a missing key will fail the same way again.
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        match self {
            Self::Transport { .. } => true,
            Self::Status { status, .. } => *status == 429 || *status >= 500,
            _ => false,
        }
    }

    /// How long the provider asked the caller to wait, if it said.
    #[must_use]
    pub const fn retry_after(&self) -> Option<Duration> {
        match self {
            Self::Status { retry_after, .. } => *retry_after,
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status(status: u16) -> LlmError {
        LlmError::Status { provider: "anthropic", status, retry_after: None, body: String::new() }
    }

    #[test]
    fn rate_limits_server_errors_and_transport_failures_are_retryable() {
        assert!(status(429).is_retryable());
        assert!(status(500).is_retryable());
        assert!(status(503).is_retryable());
        let transport =
            LlmError::Transport { provider: "gemini", message: "reset".into(), timeout: false };
        assert!(transport.is_retryable());
    }

    #[test]
    fn client_errors_and_bad_replies_are_not_retryable() {
        assert!(!status(400).is_retryable());
        assert!(!status(401).is_retryable());
        let parse = LlmError::ParseResponse { provider: "openai", message: "eof".into() };
        let schema = LlmError::Schema { provider: "openai", schema: "report", message: "x".into() };
        let key = LlmError::MissingApiKey { provider: "openai", variable: "OPENAI_API_KEY" };
        for e in [parse, schema, key, LlmError::Config("no model".into())] {
            assert!(!e.is_retryable(), "{e}");
        }
    }

    #[test]
    fn every_error_names_its_provider() {
        assert_eq!(status(500).provider(), "anthropic");
        assert_eq!(LlmError::Config(String::new()).provider(), "config");
        let key = LlmError::MissingApiKey { provider: "gemini", variable: "GEMINI_API_KEY" };
        assert_eq!(key.provider(), "gemini");
        assert!(key.to_string().contains("GEMINI_API_KEY"));
    }

    #[test]
    fn retry_after_is_only_on_a_status() {
        let limited = LlmError::Status {
            provider: "openai",
            status: 429,
            retry_after: Some(Duration::from_secs(2)),
            body: String::new(),
        };
        assert_eq!(limited.retry_after(), Some(Duration::from_secs(2)));
        assert_eq!(LlmError::Config(String::new()).retry_after(), None);
    }

    #[test]
    fn errors_are_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<LlmError>();
    }
}
