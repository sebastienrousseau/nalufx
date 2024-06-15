use serde::{Deserialize, Serialize};

/// Struct representing the response from OpenAI API.
///
/// This struct is used to deserialize and serialize the JSON response
/// from OpenAI API. It contains a vector of `OpenAIChoice` structs.
///
/// # Fields
///
/// * `choices` - A vector of `OpenAIChoice` structs representing the choices
/// provided by the OpenAI API.
#[derive(Debug, Deserialize, Serialize)]
pub struct OpenAIResponse {
    /// A vector of OpenAIChoice structs
    pub choices: Vec<OpenAIChoice>,
}

/// Struct representing a single choice from OpenAI API.
///
/// This struct is used to deserialize and serialize a single choice
/// within the JSON response from OpenAI API. It contains an `OpenAIMessage` struct.
///
/// # Fields
///
/// * `message` - An `OpenAIMessage` struct representing the message content
/// of the choice provided by the OpenAI API.
#[derive(Debug, Deserialize, Serialize)]
pub struct OpenAIChoice {
    /// An OpenAIMessage struct
    pub message: OpenAIMessage,
}

/// Struct representing a message from OpenAI API.
///
/// This struct is used to deserialize and serialize the message content
/// within a choice in the JSON response from OpenAI API.
///
/// # Fields
///
/// * `content` - A string representing the content of the message
/// provided by the OpenAI API.
#[derive(Debug, Deserialize, Serialize)]
pub struct OpenAIMessage {
    /// A string representing the content of the message
    pub content: String,
}
