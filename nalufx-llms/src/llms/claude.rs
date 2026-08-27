use crate::error::LlmError;
use crate::models::claude_dm::ClaudeResponse;
use dotenvy::dotenv;
use log::error;
use reqwest::Client;
use serde_json::Value;
use std::env;

/// Retrieves the Claude API key from the environment variables or .env file.
///
/// # Returns
///
/// * `Ok(String)` - If the API key is successfully retrieved.
/// * `Err(LlmError::MissingApiKey)` - If the API key is not present in the
///   environment or the `.env` file cannot be loaded.
pub fn get_claude_api_key() -> Result<String, LlmError> {
    // First, try to read the API key from the environment variables
    if let Ok(key) = env::var("CLAUDE_API_KEY") {
        return Ok(key);
    }

    // If the API key is not found in the environment variables, try to read it from the .env file
    match dotenv() {
        Ok(_) => match env::var("CLAUDE_API_KEY") {
            Ok(key) => Ok(key),
            Err(_) => {
                error!("CLAUDE_API_KEY not found in the .env file");
                Err(LlmError::MissingApiKey { provider: "claude" })
            },
        },
        Err(err) => {
            error!("Failed to load .env file: {:?}", err);
            Err(LlmError::MissingApiKey { provider: "claude" })
        },
    }
}

/// Sends a POST request to the Claude API with the provided request body.
///
/// # Arguments
///
/// * `client` - A reference to the reqwest::Client instance used for making HTTP requests.
/// * `api_url` - A string representing the URL of the Claude API endpoint.
/// * `api_key` - A string representing the API key for authentication.
/// * `request_body` - A serde_json::Value representing the JSON payload to be sent in the request body.
///
/// # Returns
///
/// * `Ok(String)` - If the request is successfully sent and the response body is returned as a string.
/// * `Err(LlmError::Request)` - If the transport fails or the provider
///   returns a non-success status.
pub async fn send_claude_request(
    client: &Client,
    api_url: &str,
    api_key: &str,
    request_body: Value,
) -> Result<String, LlmError> {
    let response = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await
        .map_err(|err| {
            error!("Error sending request to Claude API: {:?}", err);
            LlmError::request("claude", err.to_string())
        })?;
    if !response.status().is_success() {
        error!("Claude API call failed with status: {:?}", response.status());
        return Err(LlmError::request("claude", format!("API returned {}", response.status())));
    }
    response.text().await.map_err(|err| {
        error!("Error reading response body: {:?}", err);
        LlmError::request("claude", err.to_string())
    })
}

/// Parses the Claude API response and extracts the predictions.
///
/// # Arguments
///
/// * `body` - A string representing the JSON response from the Claude API.
///
/// # Returns
///
/// * `Ok(Vec<f64>)` - If the response is successfully parsed and the predictions are extracted.
/// * `Err(LlmError)` - If the body is not valid JSON or does not match the expected shape.
///
/// # Errors
///
/// * If the JSON response cannot be parsed into the `ClaudeResponse` struct, an error is returned with an
///   InternalServerError status and a message indicating the parsing error.
/// * If any of the prediction values cannot be parsed into a `f64`, the `unwrap_or_default` method is used
///   to provide a default value of `0.0`.
pub fn parse_claude_response(body: &str) -> Result<Vec<f64>, LlmError> {
    let claude_response: ClaudeResponse = serde_json::from_str(body).map_err(|err| {
        error!("Error parsing response JSON: {:?}", err);
        LlmError::parse("claude", err)
    })?;

    let predictions: Vec<f64> = claude_response
        .choices
        .iter()
        .flat_map(|choice| {
            choice.message.content.split_whitespace().map(|s| s.parse().unwrap_or_default())
        })
        .collect();

    Ok(predictions)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A well-formed response body carrying `content` tokens.
    fn body(contents: &[&str]) -> String {
        let choices: Vec<String> = contents
            .iter()
            .map(|c| format!(r#"{{"message":{{"content":"{c}"}}}}"#, c = c))
            .collect();
        format!(r#"{{"choices":[{}]}}"#, choices.join(","))
    }

    #[test]
    fn parses_whitespace_separated_predictions() {
        let got = parse_claude_response(&body(&["1.5 2.5 3.0"])).unwrap();
        assert_eq!(got, vec![1.5, 2.5, 3.0]);
    }

    #[test]
    fn flattens_predictions_across_choices() {
        let got = parse_claude_response(&body(&["1.0 2.0", "3.0"])).unwrap();
        assert_eq!(got, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn empty_choices_yield_no_predictions() {
        let got = parse_claude_response(r#"{"choices":[]}"#).unwrap();
        assert!(got.is_empty());
    }

    #[test]
    fn non_numeric_tokens_default_to_zero() {
        // Documented behaviour: `unwrap_or_default` substitutes 0.0 rather
        // than failing the whole batch on one unparseable token.
        let got = parse_claude_response(&body(&["1.0 not_a_number 3.0"])).unwrap();
        assert_eq!(got, vec![1.0, 0.0, 3.0]);
    }

    #[test]
    fn malformed_json_is_a_parse_error_tagged_with_the_provider() {
        let err = parse_claude_response("this is not json").unwrap_err();
        assert!(matches!(err, LlmError::ParseResponse { .. }));
        assert_eq!(err.provider(), "claude");
        assert!(!err.is_retryable(), "a bad body will not parse on retry");
    }

    #[test]
    fn a_missing_choices_field_is_a_parse_error() {
        let err = parse_claude_response(r#"{"unexpected":true}"#).unwrap_err();
        assert!(matches!(err, LlmError::ParseResponse { .. }));
    }
}
