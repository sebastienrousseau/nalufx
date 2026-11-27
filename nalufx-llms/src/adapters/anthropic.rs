//! Anthropic's Messages API (`POST /v1/messages`).

use crate::error::LlmError;
use crate::http::Http;
use crate::provider::{ChatRequest, ChatResponse, LlmConfig, LlmProvider, TokenUsage};
use crate::secret::ApiKey;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;

/// The provider's name in errors and responses.
pub const PROVIDER: &str = "anthropic";

/// The environment variable that holds the API key.
pub const API_KEY_VAR: &str = "ANTHROPIC_API_KEY";

/// The public endpoint.
pub const BASE_URL: &str = "https://api.anthropic.com";

/// The API version this adapter speaks.
pub const API_VERSION: &str = "2023-06-01";

/// A Claude model through Anthropic's Messages API.
#[derive(Clone, Debug)]
pub struct Anthropic {
    http: Http,
    base_url: String,
    api_key: ApiKey,
    config: LlmConfig,
}

impl Anthropic {
    /// An adapter for `config.model`.
    ///
    /// # Errors
    ///
    /// [`LlmError::Config`] if the configuration fails [`LlmConfig::check`].
    pub fn new(
        http: Http,
        api_key: impl Into<ApiKey>,
        config: LlmConfig,
    ) -> Result<Self, LlmError> {
        config.check()?;
        Ok(Self { http, base_url: BASE_URL.to_string(), api_key: api_key.into(), config })
    }

    /// The same adapter against another endpoint, such as a proxy or a test
    /// server.
    #[must_use]
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// The request body for the Messages API. Anthropic has no JSON mode;
    /// a JSON request is steered by the prompt and validated afterwards.
    fn body(&self, request: &ChatRequest) -> serde_json::Value {
        let mut body = json!({
            "model": self.config.model,
            "max_tokens": self.config.max_tokens,
            "messages": request.messages,
        });
        if let Some(system) = &request.system {
            body["system"] = json!(system);
        }
        if let Some(t) = self.config.temperature {
            body["temperature"] = json!(t);
        }
        body
    }
}

#[async_trait]
impl LlmProvider for Anthropic {
    fn name(&self) -> &'static str {
        PROVIDER
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    async fn complete(&self, request: &ChatRequest) -> Result<ChatResponse, LlmError> {
        let url = format!("{}/v1/messages", self.base_url.trim_end_matches('/'));
        let headers = [("x-api-key", self.api_key.expose()), ("anthropic-version", API_VERSION)];
        let reply: Reply =
            self.http.post_json(PROVIDER, &url, &headers, &self.body(request)).await?;
        reply.into_response()
    }
}

/// The parts of a Messages API reply the adapter reads.
#[derive(Debug, Deserialize)]
struct Reply {
    content: Vec<Block>,
    model: String,
    usage: Usage,
}

/// One content block; only text blocks carry the reply.
#[derive(Debug, Deserialize)]
struct Block {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

impl Reply {
    /// The text blocks joined, or an error if there are none.
    fn into_response(self) -> Result<ChatResponse, LlmError> {
        let text: String =
            self.content.into_iter().filter(|b| b.kind == "text").filter_map(|b| b.text).collect();
        if text.is_empty() {
            return Err(LlmError::ParseResponse {
                provider: PROVIDER,
                message: "the reply has no text block".to_string(),
            });
        }
        Ok(ChatResponse {
            text,
            usage: TokenUsage {
                input_tokens: self.usage.input_tokens,
                output_tokens: self.usage.output_tokens,
            },
            model: self.model,
            provider: PROVIDER.to_string(),
        })
    }
}
