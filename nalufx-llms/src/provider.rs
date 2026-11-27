//! The one provider trait, and what goes in and out of it.

use crate::error::LlmError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// A chat model behind an API.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// The provider's name, as errors and responses report it.
    fn name(&self) -> &'static str;

    /// The model the provider is configured with.
    fn model(&self) -> &str;

    /// The endpoint requests go to; a readiness check probes it.
    fn base_url(&self) -> &str;

    /// Sends one request and returns the model's reply.
    ///
    /// # Errors
    ///
    /// An [`LlmError`]: a transport failure, an error status, a reply that
    /// is not the provider's documented shape, or one with no text.
    async fn complete(&self, request: &ChatRequest) -> Result<ChatResponse, LlmError>;
}

/// Who wrote a message.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// The person or program asking.
    User,
    /// The model, in an earlier turn.
    Assistant,
}

/// One turn of a conversation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// Who wrote it.
    pub role: Role,
    /// What it says.
    pub content: String,
}

/// A request to a chat model.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChatRequest {
    /// Instructions that frame the conversation, if any.
    pub system: Option<String>,
    /// The conversation so far, oldest first; the last is usually the user's.
    pub messages: Vec<Message>,
    /// Ask the model for a JSON object (each provider's JSON mode). The
    /// caller still validates the reply: JSON mode constrains the syntax,
    /// not the schema.
    pub json: bool,
}

impl ChatRequest {
    /// A request of one user message, with optional system instructions.
    #[must_use]
    pub fn user(system: Option<&str>, prompt: impl Into<String>) -> Self {
        Self {
            system: system.map(str::to_string),
            messages: vec![Message { role: Role::User, content: prompt.into() }],
            json: false,
        }
    }
}

/// Tokens a request used, as the provider counted them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct TokenUsage {
    /// Tokens in the request.
    pub input_tokens: u32,
    /// Tokens in the reply.
    pub output_tokens: u32,
}

/// A model's reply.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatResponse {
    /// The reply's text.
    pub text: String,
    /// Tokens used.
    pub usage: TokenUsage,
    /// The model that answered, as the provider reports it.
    pub model: String,
    /// The provider that answered.
    pub provider: String,
}

/// The model and its sampling settings. The library has no default model:
/// every caller names one.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LlmConfig {
    /// The model's name at the provider, such as `claude-sonnet-5-5`.
    pub model: String,
    /// The most tokens the reply may have.
    pub max_tokens: u32,
    /// The sampling temperature; the provider's default if `None`.
    pub temperature: Option<f64>,
}

impl LlmConfig {
    /// A configuration for `model` with `max_tokens` and the provider's
    /// default temperature.
    ///
    /// # Errors
    ///
    /// [`LlmError::Config`] if the model name is blank or `max_tokens` is 0.
    pub fn new(model: impl Into<String>, max_tokens: u32) -> Result<Self, LlmError> {
        let config = Self { model: model.into(), max_tokens, temperature: None };
        config.check()?;
        Ok(config)
    }

    /// Checks the configuration.
    ///
    /// # Errors
    ///
    /// [`LlmError::Config`] if the model name is blank, `max_tokens` is 0,
    /// or the temperature is negative or not finite.
    pub fn check(&self) -> Result<(), LlmError> {
        if self.model.trim().is_empty() {
            return Err(LlmError::Config("the model name is empty".to_string()));
        }
        if self.max_tokens == 0 {
            return Err(LlmError::Config("max_tokens must be at least 1".to_string()));
        }
        if self.temperature.is_some_and(|t| !(t.is_finite() && t >= 0.0)) {
            return Err(LlmError::Config(
                "the temperature must be finite and not negative".to_string(),
            ));
        }
        Ok(())
    }
}

/// How the shared HTTP client and the retries behave.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientConfig {
    /// The longest wait for a connection.
    pub connect_timeout: Duration,
    /// The longest wait for a whole request, connection included.
    pub request_timeout: Duration,
    /// Attempts per request, the first included.
    pub max_attempts: u32,
    /// The wait before the first retry; it doubles each time.
    pub base_delay: Duration,
    /// The longest wait between attempts, `Retry-After` included.
    pub max_delay: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(60),
            max_attempts: 3,
            base_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_config_needs_a_model_and_tokens() {
        assert!(LlmConfig::new("claude-sonnet-5-5", 1024).is_ok());
        assert!(matches!(LlmConfig::new("  ", 1024), Err(LlmError::Config(_))));
        assert!(matches!(LlmConfig::new("m", 0), Err(LlmError::Config(_))));
        let mut hot = LlmConfig::new("m", 10).unwrap();
        hot.temperature = Some(-0.1);
        assert!(matches!(hot.check(), Err(LlmError::Config(_))));
        hot.temperature = Some(f64::NAN);
        assert!(hot.check().is_err());
        hot.temperature = Some(0.2);
        assert!(hot.check().is_ok());
    }

    #[test]
    fn the_client_defaults_are_the_planned_ones() {
        let c = ClientConfig::default();
        assert_eq!(
            (c.connect_timeout, c.request_timeout, c.max_attempts),
            (Duration::from_secs(5), Duration::from_secs(60), 3)
        );
    }

    #[test]
    fn a_user_request_has_one_message() {
        let r = ChatRequest::user(Some("be brief"), "hello");
        assert_eq!(r.system.as_deref(), Some("be brief"));
        assert_eq!(r.messages, vec![Message { role: Role::User, content: "hello".into() }]);
        assert!(!r.json);
    }
}
