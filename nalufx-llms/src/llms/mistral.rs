use crate::error::LlmError;
use crate::models::mistral_dm::MistralResponse;
use dotenvy::dotenv;
use log::error;
use reqwest::Client;
use std::env;

/// Retrieves the Mistral API key from the environment variables or .env file.
///
/// # Returns
///
/// * `Ok(String)` - If the API key is successfully retrieved.
/// * `Err(LlmError::MissingApiKey)` - If the API key is not present in the
///   environment or the `.env` file cannot be loaded.
pub fn get_mistral_api_key() -> Result<String, LlmError> {
    // First, try to read the API key from the environment variables
    if let Ok(key) = env::var("MISTRAL_API_KEY") {
        return Ok(key);
    }

    // If the API key is not found in the environment variables, try to read it from the .env file
    match dotenv() {
        Ok(_) => match env::var("MISTRAL_API_KEY") {
            Ok(key) => Ok(key),
            Err(_) => {
                error!("MISTRAL_API_KEY not found in the .env file");
                Err(LlmError::MissingApiKey { provider: "mistral" })
            },
        },
        Err(err) => {
            error!("Failed to load .env file: {:?}", err);
            Err(LlmError::MissingApiKey { provider: "mistral" })
        },
    }
}

/// Sends a POST request to the Mistral API with the provided request body.
///
/// # Arguments
///
/// * `client` - A reference to the reqwest::Client instance used for making HTTP requests.
/// * `api_url` - A string representing the URL of the Mistral API endpoint.
/// * `api_key` - A string representing the API key for authentication.
/// * `request_body` - A serde_json::Value representing the JSON payload to be sent in the request body.
///
/// # Returns
///
/// * `Ok(serde_json::Value)` - If the request is successfully sent and the response body is returned as a serde_json::Value.
/// * `Err(reqwest::Error)` - If an error occurs during the request or response handling.
pub async fn send_mistral_request(
    client: &Client,
    api_url: &str,
    api_key: &str,
    request_body: serde_json::Value,
) -> Result<serde_json::Value, reqwest::Error> {
    let response = client.post(api_url).bearer_auth(api_key).json(&request_body).send().await?;

    let json_response: serde_json::Value = response.json().await?;
    Ok(json_response)
}

/// Parses the Mistral API response and extracts the predictions.
///
/// # Arguments
///
/// * `body` - A string representing the JSON response from the Mistral API.
///
/// # Returns
///
/// * `Ok(Vec<f64>)` - If the response is successfully parsed and the predictions are extracted.
/// * `Err(LlmError)` - If the body is not valid JSON or does not match the expected shape.
///
/// # Errors
///
/// * If the JSON response cannot be parsed into the `MistralResponse` struct, an error is returned with an
///   InternalServerError status and a message indicating the parsing error.
/// * If any of the prediction values cannot be parsed into a `f64`, the `unwrap_or_default` method is used
///   to provide a default value of `0.0`.
pub fn parse_mistral_response(body: &str) -> Result<Vec<f64>, LlmError> {
    let mistral_response: MistralResponse = serde_json::from_str(body).map_err(|err| {
        error!("Error parsing response JSON: {:?}", err);
        LlmError::parse("mistral", err)
    })?;

    let predictions: Vec<f64> = mistral_response
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
        let got = parse_mistral_response(&body(&["1.5 2.5 3.0"])).unwrap();
        assert_eq!(got, vec![1.5, 2.5, 3.0]);
    }

    #[test]
    fn flattens_predictions_across_choices() {
        let got = parse_mistral_response(&body(&["1.0 2.0", "3.0"])).unwrap();
        assert_eq!(got, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn empty_choices_yield_no_predictions() {
        let got = parse_mistral_response(r#"{"choices":[]}"#).unwrap();
        assert!(got.is_empty());
    }

    #[test]
    fn non_numeric_tokens_default_to_zero() {
        // Documented behaviour: `unwrap_or_default` substitutes 0.0 rather
        // than failing the whole batch on one unparseable token.
        let got = parse_mistral_response(&body(&["1.0 not_a_number 3.0"])).unwrap();
        assert_eq!(got, vec![1.0, 0.0, 3.0]);
    }

    #[test]
    fn malformed_json_is_a_parse_error_tagged_with_the_provider() {
        let err = parse_mistral_response("this is not json").unwrap_err();
        assert!(matches!(err, LlmError::ParseResponse { .. }));
        assert_eq!(err.provider(), "mistral");
        assert!(!err.is_retryable(), "a bad body will not parse on retry");
    }

    #[test]
    fn a_missing_choices_field_is_a_parse_error() {
        let err = parse_mistral_response(r#"{"unexpected":true}"#).unwrap_err();
        assert!(matches!(err, LlmError::ParseResponse { .. }));
    }
}
