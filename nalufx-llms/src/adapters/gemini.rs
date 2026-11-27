//! Google's Gemini API (`POST /v1beta/models/{model}:generateContent`).

use crate::error::LlmError;
use crate::http::Http;
use crate::provider::{ChatRequest, ChatResponse, LlmConfig, LlmProvider, Role, TokenUsage};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;

/// The provider's name in errors and responses.
pub const PROVIDER: &str = "gemini";

/// The environment variable that holds the API key.
pub const API_KEY_VAR: &str = "GEMINI_API_KEY";

/// The public endpoint.
pub const BASE_URL: &str = "https://generativelanguage.googleapis.com";

/// A Gemini model through the `generateContent` API.
#[derive(Clone, Debug)]
pub struct Gemini {
    http: Http,
    base_url: String,
    api_key: String,
    config: LlmConfig,
}

impl Gemini {
    /// An adapter for `config.model`.
    ///
    /// # Errors
    ///
    /// [`LlmError::Config`] if the configuration fails [`LlmConfig::check`].
    pub fn new(
        http: Http,
        api_key: impl Into<String>,
        config: LlmConfig,
    ) -> Result<Self, LlmError> {
        config.check()?;
        Ok(Self { http, base_url: BASE_URL.to_string(), api_key: api_key.into(), config })
    }

    /// The same adapter against another endpoint, such as a test server.
    #[must_use]
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// The request body. Gemini calls the assistant role `model`.
    fn body(&self, request: &ChatRequest) -> serde_json::Value {
        let contents: Vec<_> = request
            .messages
            .iter()
            .map(|m| {
                let role = if m.role == Role::User { "user" } else { "model" };
                json!({ "role": role, "parts": [{ "text": m.content }] })
            })
            .collect();
        let mut generation = json!({ "maxOutputTokens": self.config.max_tokens });
        if let Some(t) = self.config.temperature {
            generation["temperature"] = json!(t);
        }
        if request.json {
            generation["responseMimeType"] = json!("application/json");
        }
        let mut body = json!({ "contents": contents, "generationConfig": generation });
        if let Some(system) = &request.system {
            body["systemInstruction"] = json!({ "parts": [{ "text": system }] });
        }
        body
    }
}

#[async_trait]
impl LlmProvider for Gemini {
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
        let url = format!(
            "{}/v1beta/models/{}:generateContent",
            self.base_url.trim_end_matches('/'),
            self.config.model
        );
        let headers = [("x-goog-api-key", self.api_key.as_str())];
        let reply: Reply =
            self.http.post_json(PROVIDER, &url, &headers, &self.body(request)).await?;
        reply.into_response(&self.config.model)
    }
}

/// The parts of a `generateContent` reply the adapter reads.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Reply {
    #[serde(default)]
    candidates: Vec<Candidate>,
    #[serde(default)]
    usage_metadata: Option<Usage>,
    #[serde(default)]
    model_version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: Content,
}

#[derive(Debug, Deserialize)]
struct Content {
    #[serde(default)]
    parts: Vec<Part>,
}

#[derive(Debug, Deserialize)]
struct Part {
    #[serde(default)]
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Usage {
    #[serde(default)]
    prompt_token_count: u32,
    #[serde(default)]
    candidates_token_count: u32,
}

impl Reply {
    /// The first candidate's text parts joined, or an error if there is no
    /// text (Gemini returns no candidate when it blocks a prompt).
    fn into_response(self, requested_model: &str) -> Result<ChatResponse, LlmError> {
        let text: String = self
            .candidates
            .into_iter()
            .next()
            .map(|c| c.content.parts.into_iter().filter_map(|p| p.text).collect())
            .unwrap_or_default();
        if text.is_empty() {
            return Err(LlmError::ParseResponse {
                provider: PROVIDER,
                message: "the reply has no candidate text".to_string(),
            });
        }
        let usage = self.usage_metadata.map_or_else(TokenUsage::default, |u| TokenUsage {
            input_tokens: u.prompt_token_count,
            output_tokens: u.candidates_token_count,
        });
        Ok(ChatResponse {
            text,
            usage,
            model: self.model_version.unwrap_or_else(|| requested_model.to_string()),
            provider: PROVIDER.to_string(),
        })
    }
}
