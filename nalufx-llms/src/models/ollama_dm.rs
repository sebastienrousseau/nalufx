use serde::{Deserialize, Serialize};

/// Struct representing the response from Ollama API.
///
/// This struct is used to deserialize and serialize the JSON response
/// from Ollama API. It contains a vector of `OllamaChoice` structs.
///
/// # Fields
///
/// * `choices` - A vector of `OllamaChoice` structs representing the choices
/// provided by the Ollama API.
#[derive(Debug, Deserialize, Serialize)]
pub struct OllamaResponse {
    /// A vector of OllamaChoice structs
    pub choices: Vec<OllamaChoice>,
}

/// Struct representing a single choice from Ollama API.
///
/// This struct is used to deserialize and serialize a single choice
/// within the JSON response from Ollama API. It contains a `OllamaMessage` struct.
///
/// # Fields
///
/// * `message` - A `OllamaMessage` struct representing the message content
/// of the choice provided by the Ollama API.
#[derive(Debug, Deserialize, Serialize)]
pub struct OllamaChoice {
    /// A OllamaMessage struct
    pub message: OllamaMessage,
}

/// Struct representing a message from Ollama API.
///
/// This struct is used to deserialize and serialize the message content
/// within a choice in the JSON response from Ollama API.
///
/// # Fields
///
/// * `content` - A string representing the content of the message
/// provided by the Ollama API.
#[derive(Debug, Deserialize, Serialize)]
pub struct OllamaMessage {
    /// A string representing the content of the message
    pub content: String,
}
