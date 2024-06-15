use serde::{Deserialize, Serialize};

/// Struct representing the response from Mistral API.
///
/// This struct is used to deserialize and serialize the JSON response
/// from Mistral API. It contains a vector of `MistralChoice` structs.
///
/// # Fields
///
/// * `choices` - A vector of `MistralChoice` structs representing the choices
/// provided by the Mistral API.
#[derive(Debug, Deserialize, Serialize)]
pub struct MistralResponse {
    /// A vector of MistralChoice structs
    pub choices: Vec<MistralChoice>,
}

/// Struct representing a single choice from Mistral API.
///
/// This struct is used to deserialize and serialize a single choice
/// within the JSON response from Mistral API. It contains a `MistralMessage` struct.
///
/// # Fields
///
/// * `message` - A `MistralMessage` struct representing the message content
/// of the choice provided by the Mistral API.
#[derive(Debug, Deserialize, Serialize)]
pub struct MistralChoice {
    /// A MistralMessage struct
    pub message: MistralMessage,
}

/// Struct representing a message from Mistral API.
///
/// This struct is used to deserialize and serialize the message content
/// within a choice in the JSON response from Mistral API.
///
/// # Fields
///
/// * `content` - A string representing the content of the message
/// provided by the Mistral API.
#[derive(Debug, Deserialize, Serialize)]
pub struct MistralMessage {
    /// A string representing the content of the message
    pub content: String,
}
