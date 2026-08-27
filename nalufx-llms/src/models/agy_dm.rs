use serde::{Deserialize, Serialize};

/// Struct representing the response from Agy API.
///
/// This struct is used to deserialize and serialize the JSON response
/// from Agy API. It contains a vector of `AgyChoice` structs.
///
/// # Fields
///
/// * `choices` - A vector of `AgyChoice` structs representing the choices
///   provided by the Agy API.
#[derive(Debug, Deserialize, Serialize)]
pub struct AgyResponse {
    /// A vector of AgyChoice structs
    pub choices: Vec<AgyChoice>,
}

/// Struct representing a single choice from Agy API.
///
/// This struct is used to deserialize and serialize a single choice
/// within the JSON response from Agy API. It contains a `AgyMessage` struct.
///
/// # Fields
///
/// * `message` - A `AgyMessage` struct representing the message content
///   of the choice provided by the Agy API.
#[derive(Debug, Deserialize, Serialize)]
pub struct AgyChoice {
    /// A AgyMessage struct
    pub message: AgyMessage,
}

/// Struct representing a message from Agy API.
///
/// This struct is used to deserialize and serialize the message content
/// within a choice in the JSON response from Agy API.
///
/// # Fields
///
/// * `content` - A string representing the content of the message
///   provided by the Agy API.
#[derive(Debug, Deserialize, Serialize)]
pub struct AgyMessage {
    /// A string representing the content of the message
    pub content: String,
}
