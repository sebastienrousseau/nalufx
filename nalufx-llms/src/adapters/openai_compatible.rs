//! Every API that speaks OpenAI's chat completions
//! (`POST {base}/chat/completions`): OpenAI, Mistral, Ollama, Agy, and
//! Llama servers such as vLLM or llama.cpp.

use crate::error::LlmError;
use crate::http::Http;
use crate::provider::{ChatRequest, ChatResponse, LlmConfig, LlmProvider, TokenUsage};
use crate::secret::ApiKey;
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;

/// A known OpenAI-compatible endpoint: its name, base URL and key variable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Endpoint {
    /// The provider's name in errors and responses.
    pub provider: &'static str,
    /// The base URL, up to and including `/v1`; `None` where each
    /// deployment has its own.
    pub base_url: Option<&'static str>,
    /// The environment variable that holds the API key; `None` where the
    /// server needs none.
    pub api_key_var: Option<&'static str>,
}

/// OpenAI.
pub const OPENAI: Endpoint = Endpoint {
    provider: "openai",
    base_url: Some("https://api.openai.com/v1"),
    api_key_var: Some("OPENAI_API_KEY"),
};
/// Mistral.
pub const MISTRAL: Endpoint = Endpoint {
    provider: "mistral",
    base_url: Some("https://api.mistral.ai/v1"),
    api_key_var: Some("MISTRAL_API_KEY"),
};
/// A local Ollama server.
pub const OLLAMA: Endpoint =
    Endpoint { provider: "ollama", base_url: Some("http://localhost:11434/v1"), api_key_var: None };
/// Agy, whose endpoint each deployment supplies.
pub const AGY: Endpoint =
    Endpoint { provider: "agy", base_url: None, api_key_var: Some("AGY_API_KEY") };
/// Any other OpenAI-compatible server, such as vLLM or llama.cpp serving Llama.
pub const CUSTOM: Endpoint =
    Endpoint { provider: "openai_compatible", base_url: None, api_key_var: None };

/// A model behind an OpenAI-compatible chat completions API.
#[derive(Clone, Debug)]
pub struct OpenAiCompatible {
    http: Http,
    provider: &'static str,
    base_url: String,
    api_key: Option<ApiKey>,
    config: LlmConfig,
}

impl OpenAiCompatible {
    /// An adapter for `config.model` at `base_url` (up to and including
    /// `/v1`), sending `api_key` as a bearer token if there is one.
    ///
    /// # Errors
    ///
    /// [`LlmError::Config`] if the configuration fails [`LlmConfig::check`]
    /// or the base URL is not `http://` or `https://`.
    pub fn new(
        http: Http,
        endpoint: Endpoint,
        base_url: impl Into<String>,
        api_key: Option<ApiKey>,
        config: LlmConfig,
    ) -> Result<Self, LlmError> {
        config.check()?;
        let base_url = base_url.into();
        if !(base_url.starts_with("http://") || base_url.starts_with("https://")) {
            return Err(LlmError::Config(format!("`{base_url}` is not an http(s) URL")));
        }
        Ok(Self { http, provider: endpoint.provider, base_url, api_key, config })
    }

    /// The request body, with the system instructions as the first message.
    fn body(&self, request: &ChatRequest) -> serde_json::Value {
        let system = request.system.iter().map(|s| json!({ "role": "system", "content": s }));
        let rest = request.messages.iter().map(|m| json!({ "role": m.role, "content": m.content }));
        let mut body = json!({
            "model": self.config.model,
            "max_tokens": self.config.max_tokens,
            "messages": system.chain(rest).collect::<Vec<_>>(),
        });
        if let Some(t) = self.config.temperature {
            body["temperature"] = json!(t);
        }
        if request.json {
            body["response_format"] = json!({ "type": "json_object" });
        }
        body
    }
}

#[async_trait]
impl LlmProvider for OpenAiCompatible {
    fn name(&self) -> &'static str {
        self.provider
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    async fn complete(&self, request: &ChatRequest) -> Result<ChatResponse, LlmError> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let bearer = self.api_key.as_ref().map(|k| format!("Bearer {}", k.expose()));
        let headers: Vec<(&str, &str)> =
            bearer.iter().map(|b| ("authorization", b.as_str())).collect();
        let reply: Reply =
            self.http.post_json(self.provider, &url, &headers, &self.body(request)).await?;
        reply.into_response(self.provider, &self.config.model)
    }
}

/// The parts of a chat completions reply the adapter reads.
#[derive(Debug, Deserialize)]
struct Reply {
    choices: Vec<Choice>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ReplyMessage,
}

#[derive(Debug, Deserialize)]
struct ReplyMessage {
    #[serde(default)]
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    #[serde(default)]
    prompt_tokens: u32,
    #[serde(default)]
    completion_tokens: u32,
}

impl Reply {
    /// The first choice's text, or an error if there is none.
    fn into_response(
        self,
        provider: &'static str,
        requested_model: &str,
    ) -> Result<ChatResponse, LlmError> {
        let text =
            self.choices.into_iter().next().and_then(|c| c.message.content).unwrap_or_default();
        if text.is_empty() {
            return Err(LlmError::ParseResponse {
                provider,
                message: "the reply has no message content".to_string(),
            });
        }
        let usage = self.usage.map_or_else(TokenUsage::default, |u| TokenUsage {
            input_tokens: u.prompt_tokens,
            output_tokens: u.completion_tokens,
        });
        let model = self.model.unwrap_or_else(|| requested_model.to_string());
        Ok(ChatResponse { text, usage, model, provider: provider.to_string() })
    }
}
