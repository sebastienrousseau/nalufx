use serde::{Deserialize, Serialize};

/// Struct representing the response from Gemma API.
///
/// This struct is used to deserialize and serialize the JSON response
/// from Gemma API. It contains a vector of `GemmaChoice` structs.
///
/// # Fields
///
/// * `choices` - A vector of `GemmaChoice` structs representing the choices
/// provided by the Gemma API.
#[derive(Debug, Deserialize, Serialize)]
pub struct GemmaResponse {
    /// A vector of GemmaChoice structs
    pub choices: Vec<GemmaChoice>,
}

/// Struct representing a single choice from Gemma API.
///
/// This struct is used to deserialize and serialize a single choice
/// within the JSON response from Gemma API. It contains a `GemmaMessage` struct.
///
/// # Fields
///
/// * `message` - A `GemmaMessage` struct representing the message content
/// of the choice provided by the Gemma API.
#[derive(Debug, Deserialize, Serialize)]
pub struct GemmaChoice {
    /// A GemmaMessage struct
    pub message: GemmaMessage,
}

/// Struct representing a message from Gemma API.
///
/// This struct is used to deserialize and serialize the message content
/// within a choice in the JSON response from Gemma API.
///
/// # Fields
///
/// * `content` - A string representing the content of the message
/// provided by the Gemma API.
#[derive(Debug, Deserialize, Serialize)]
pub struct GemmaMessage {
    /// A string representing the content of the message
    pub content: String,
}
