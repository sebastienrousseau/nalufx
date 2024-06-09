use serde::{Deserialize, Serialize};

/// Struct representing the response from Claude API.
///
/// This struct is used to deserialize and serialize the JSON response
/// from Claude API. It contains a vector of `ClaudeChoice` structs.
///
/// # Fields
///
/// * `choices` - A vector of `ClaudeChoice` structs representing the choices
/// provided by the Claude API.
#[derive(Debug, Deserialize, Serialize)]
pub struct ClaudeResponse {
    /// A vector of ClaudeChoice structs
    pub choices: Vec<ClaudeChoice>,
}

/// Struct representing a single choice from Claude API.
///
/// This struct is used to deserialize and serialize a single choice
/// within the JSON response from Claude API. It contains a `ClaudeMessage` struct.
///
/// # Fields
///
/// * `message` - A `ClaudeMessage` struct representing the message content
/// of the choice provided by the Claude API.
#[derive(Debug, Deserialize, Serialize)]
pub struct ClaudeChoice {
    /// A ClaudeMessage struct
    pub message: ClaudeMessage,
}

/// Struct representing a message from Claude API.
///
/// This struct is used to deserialize and serialize the message content
/// within a choice in the JSON response from Claude API.
///
/// # Fields
///
/// * `content` - A string representing the content of the message
/// provided by the Claude API.
#[derive(Debug, Deserialize, Serialize)]
pub struct ClaudeMessage {
    /// A string representing the content of the message
    pub content: String,
}
