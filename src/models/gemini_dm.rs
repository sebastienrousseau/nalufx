use serde::{Deserialize, Serialize};

/// Struct representing the response from Gemini API.
///
/// This struct is used to deserialize and serialize the JSON response
/// from Gemini API. It contains a vector of `GeminiChoice` structs.
///
/// # Fields
///
/// * `choices` - A vector of `GeminiChoice` structs representing the choices
/// provided by the Gemini API.
#[derive(Debug, Deserialize, Serialize)]
pub struct GeminiResponse {
    /// A vector of GeminiChoice structs
    pub choices: Vec<GeminiChoice>,
}

/// Struct representing a single choice from Gemini API.
///
/// This struct is used to deserialize and serialize a single choice
/// within the JSON response from Gemini API. It contains a `GeminiMessage` struct.
///
/// # Fields
///
/// * `message` - A `GeminiMessage` struct representing the message content
/// of the choice provided by the Gemini API.
#[derive(Debug, Deserialize, Serialize)]
pub struct GeminiChoice {
    /// A GeminiMessage struct
    pub message: GeminiMessage,
}

/// Struct representing a message from Gemini API.
///
/// This struct is used to deserialize and serialize the message content
/// within a choice in the JSON response from Gemini API.
///
/// # Fields
///
/// * `content` - A string representing the content of the message
/// provided by the Gemini API.
#[derive(Debug, Deserialize, Serialize)]
pub struct GeminiMessage {
    /// A string representing the content of the message
    pub content: String,
}
