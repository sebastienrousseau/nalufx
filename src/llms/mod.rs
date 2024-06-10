use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

/// A trait representing a Language Model (LLM) with a method to send requests.
///
/// This trait is used to define the common interface for different LLM APIs.
///
#[async_trait]
pub trait LLM: Sync + Send {
    /// Sends a request to the LLM API.
    ///
    /// # Arguments
    ///
    /// * `client` - A reference to the `reqwest::Client` used to make the request.
    /// * `api_key` - A reference to the API key used for authentication.
    /// * `prompt` - A reference to the prompt to be sent to the LLM.
    /// * `max_tokens` - The maximum number of tokens allowed in the response.
    ///
    /// # Returns
    ///
    /// * `Result<Value, reqwest::Error>` - A `Result` containing the JSON response from the LLM API
    ///   on success, or a `reqwest::Error` on failure.
    ///
    async fn send_request(
        &self,
        client: &Client,
        api_key: &str,
        prompt: &str,
        max_tokens: usize,
    ) -> Result<Value, reqwest::Error>;
}

/// This module contains the Claude API handlers.
pub mod claude;

/// This module contains the Gemini API handlers.
pub mod gemini;

/// This module contains the Mistral API handlers.
pub mod mistral;

/// This module contains the Ollama API handlers.
pub mod ollama;

/// This module contains the OpenAI API handlers.
pub mod openai;
