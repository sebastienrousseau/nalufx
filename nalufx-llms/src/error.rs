// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Errors produced by the LLM adapters.
//!
//! These are deliberately transport-agnostic. The adapters previously
//! returned `actix_web::HttpResponse`, which meant a library crate
//! decided the HTTP status of a failure and could not be used outside
//! an actix application at all. Mapping a domain error to a response is
//! the caller's job, and lives in `nalufx-core`.

use thiserror::Error;

/// A failure while talking to, or interpreting a response from, an LLM.
#[derive(Debug, Error)]
pub enum LlmError {
    /// The response body was not valid JSON, or did not match the shape
    /// this provider is documented to return.
    #[error("could not parse the {provider} response: {source}")]
    ParseResponse {
        /// Provider whose response failed to parse.
        provider: &'static str,
        /// The underlying `serde_json` failure.
        #[source]
        source: serde_json::Error,
    },

    /// The request never reached the provider, or the transport failed.
    #[error("request to {provider} failed: {message}")]
    Request {
        /// Provider the request was addressed to.
        provider: &'static str,
        /// What went wrong.
        message: String,
    },

    /// A required credential was absent or empty.
    #[error("missing API key for {provider}")]
    MissingApiKey {
        /// Provider whose key is required.
        provider: &'static str,
    },
}

impl LlmError {
    /// Builds a [`LlmError::ParseResponse`] for `provider`.
    pub const fn parse(provider: &'static str, source: serde_json::Error) -> Self {
        Self::ParseResponse { provider, source }
    }

    /// Builds a [`LlmError::Request`] for `provider`.
    pub fn request(provider: &'static str, message: impl Into<String>) -> Self {
        Self::Request { provider, message: message.into() }
    }

    /// The provider this error came from.
    #[must_use]
    pub const fn provider(&self) -> &'static str {
        match self {
            Self::ParseResponse { provider, .. }
            | Self::Request { provider, .. }
            | Self::MissingApiKey { provider } => provider,
        }
    }

    /// Whether retrying the same call could plausibly succeed.
    ///
    /// A transport failure may be transient. A malformed response or a
    /// missing key will not fix itself.
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        matches!(self, Self::Request { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::LlmError;

    fn json_error() -> serde_json::Error {
        serde_json::from_str::<serde_json::Value>("{").unwrap_err()
    }

    #[test]
    fn parse_error_names_the_provider() {
        let e = LlmError::parse("openai", json_error());
        assert_eq!(e.provider(), "openai");
        assert!(e.to_string().contains("openai"));
        assert!(!e.is_retryable());
    }

    #[test]
    fn request_error_is_retryable() {
        let e = LlmError::request("claude", "connection reset");
        assert_eq!(e.provider(), "claude");
        assert!(e.is_retryable());
        assert!(e.to_string().contains("connection reset"));
    }

    #[test]
    fn missing_key_is_not_retryable() {
        let e = LlmError::MissingApiKey { provider: "gemini" };
        assert_eq!(e.provider(), "gemini");
        assert!(!e.is_retryable());
    }
}
