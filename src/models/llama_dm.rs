use serde::{Deserialize, Serialize};

/// Struct representing the response from Llama API.
///
/// This struct is used to deserialize and serialize the JSON response
/// from Llama API. It contains a vector of `LlamaChoice` structs.
///
/// # Fields
///
/// * `choices` - A vector of `LlamaChoice` structs representing the choices
/// provided by the Llama API.
#[derive(Debug, Deserialize, Serialize)]
pub struct LlamaResponse {
    /// A vector of LlamaChoice structs
    pub choices: Vec<LlamaChoice>,
}

/// Struct representing a single choice from Llama API.
///
/// This struct is used to deserialize and serialize a single choice
/// within the JSON response from Llama API. It contains a `LlamaMessage` struct.
///
/// # Fields
///
/// * `message` - A `LlamaMessage` struct representing the message content
/// of the choice provided by the Llama API.
#[derive(Debug, Deserialize, Serialize)]
pub struct LlamaChoice {
    /// A LlamaMessage struct
    pub message: LlamaMessage,
}

/// Struct representing a message from Llama API.
///
/// This struct is used to deserialize and serialize the message content
/// within a choice in the JSON response from Llama API.
///
/// # Fields
///
/// * `content` - A string representing the content of the message
/// provided by the Llama API.
#[derive(Debug, Deserialize, Serialize)]
pub struct LlamaMessage {
    /// A string representing the content of the message
    pub content: String,
}
