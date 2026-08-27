//! Adapter for the Agy chat-completions API.
//!
//! The endpoint is supplied by the caller rather than hardcoded here,
//! matching the other adapters in this module: the crate speaks the
//! protocol, and deployment decides where.

use crate::error::LlmError;
use crate::models::agy_dm::AgyResponse;
use dotenvy::dotenv;
use log::error;
use reqwest::Client;
use std::env;

/// The environment variable holding the Agy API key.
pub const API_KEY_VAR: &str = "AGY_API_KEY";

/// The provider name carried on every [`LlmError`] this module returns.
pub const PROVIDER: &str = "agy";

/// Retrieves the Agy API key from the environment or a `.env` file.
///
/// The environment is checked first so a deployment can override
/// whatever `.env` happens to be on disk.
///
/// # Returns
///
/// * `Ok(String)` - If the API key is successfully retrieved.
/// * `Err(LlmError::MissingApiKey)` - If the API key is not present in
///   the environment or the `.env` file cannot be loaded.
///
/// # Errors
///
/// Returns [`LlmError::MissingApiKey`] rather than distinguishing
/// "absent" from "`.env` unreadable": both leave the caller without a
/// credential, and the distinction is only useful in the log.
///
/// # Examples
///
/// ```no_run
/// use nalufx_llms::llms::agy::get_agy_api_key;
///
/// match get_agy_api_key() {
///     Ok(key) => println!("key is {} characters", key.len()),
///     Err(err) => eprintln!("{err}"),
/// }
/// ```
pub fn get_agy_api_key() -> Result<String, LlmError> {
    // Check the environment first; it takes precedence over `.env`.
    if let Ok(key) = env::var(API_KEY_VAR) {
        return Ok(key);
    }

    match dotenv() {
        Ok(_) => match env::var(API_KEY_VAR) {
            Ok(key) => Ok(key),
            Err(_) => {
                error!("{API_KEY_VAR} not found in the .env file");
                Err(LlmError::MissingApiKey { provider: PROVIDER })
            },
        },
        Err(err) => {
            error!("Failed to load .env file: {err:?}");
            Err(LlmError::MissingApiKey { provider: PROVIDER })
        },
    }
}

/// Sends a POST request to the Agy API and returns the raw body.
///
/// The body is returned as a string rather than parsed here so that
/// [`parse_agy_response`] stays a pure function, testable without a
/// network.
///
/// # Arguments
///
/// * `client` - The HTTP client used for the request.
/// * `api_url` - The Agy chat-completions endpoint.
/// * `api_key` - The bearer token.
/// * `request_body` - The JSON payload.
///
/// # Returns
///
/// * `Ok(String)` - The response body, on a success status.
/// * `Err(LlmError::Request)` - On a transport failure or a non-success
///   status.
///
/// # Errors
///
/// Returns [`LlmError::Request`], which reports `is_retryable() == true`
/// so a caller can back off and try again.
pub async fn send_agy_request(
    client: &Client,
    api_url: &str,
    api_key: &str,
    request_body: serde_json::Value,
) -> Result<String, LlmError> {
    let response =
        client.post(api_url).bearer_auth(api_key).json(&request_body).send().await.map_err(
            |err| {
                error!("Error sending request to Agy: {err:?}");
                LlmError::request(PROVIDER, err.to_string())
            },
        )?;

    if !response.status().is_success() {
        return Err(LlmError::request(PROVIDER, format!("API returned {}", response.status())));
    }

    response.text().await.map_err(|err| {
        error!("Error reading response body: {err:?}");
        LlmError::request(PROVIDER, err.to_string())
    })
}

/// Parses an Agy API response and extracts the predictions.
///
/// # Arguments
///
/// * `body` - The JSON response from the Agy API.
///
/// # Returns
///
/// * `Ok(Vec<f64>)` - The whitespace-separated values from every choice,
///   flattened in order.
/// * `Err(LlmError::ParseResponse)` - If the body is not valid JSON or
///   does not match the expected shape.
///
/// # Errors
///
/// Returns [`LlmError::ParseResponse`] when the body will not
/// deserialise. A token that is not a number does *not* fail the batch:
/// it becomes `0.0`, matching the sibling adapters, so one malformed
/// value cannot discard an otherwise usable completion.
///
/// # Examples
///
/// ```
/// use nalufx_llms::llms::agy::parse_agy_response;
///
/// let body = r#"{"choices":[{"message":{"content":"1.5 2.5 3.0"}}]}"#;
/// assert_eq!(parse_agy_response(body).unwrap(), vec![1.5, 2.5, 3.0]);
/// ```
pub fn parse_agy_response(body: &str) -> Result<Vec<f64>, LlmError> {
    let agy_response: AgyResponse = serde_json::from_str(body).map_err(|err| {
        error!("Error parsing response JSON: {err:?}");
        LlmError::parse(PROVIDER, err)
    })?;

    let predictions: Vec<f64> = agy_response
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
        let choices: Vec<String> =
            contents.iter().map(|c| format!(r#"{{"message":{{"content":"{c}"}}}}"#)).collect();
        format!(r#"{{"choices":[{}]}}"#, choices.join(","))
    }

    #[test]
    fn parses_whitespace_separated_predictions() {
        let got = parse_agy_response(&body(&["1.5 2.5 3.0"])).unwrap();
        assert_eq!(got, vec![1.5, 2.5, 3.0]);
    }

    #[test]
    fn flattens_predictions_across_choices() {
        let got = parse_agy_response(&body(&["1.0 2.0", "3.0"])).unwrap();
        assert_eq!(got, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn empty_choices_yield_no_predictions() {
        let got = parse_agy_response(r#"{"choices":[]}"#).unwrap();
        assert!(got.is_empty());
    }

    #[test]
    fn non_numeric_tokens_default_to_zero() {
        // Documented behaviour: `unwrap_or_default` substitutes 0.0
        // rather than failing the whole batch on one bad token.
        let got = parse_agy_response(&body(&["1.0 not_a_number 3.0"])).unwrap();
        assert_eq!(got, vec![1.0, 0.0, 3.0]);
    }

    #[test]
    fn negative_and_exponent_forms_parse() {
        let got = parse_agy_response(&body(&["-1.5 2e3 0.0"])).unwrap();
        assert_eq!(got, vec![-1.5, 2000.0, 0.0]);
    }

    #[test]
    fn irregular_whitespace_is_tolerated() {
        // `split_whitespace` collapses runs and handles tabs/newlines,
        // so a prettily formatted completion still parses.
        let got = parse_agy_response(&body(&["1.0\\t2.0\\n  3.0"])).unwrap();
        assert_eq!(got, vec![1.0, 2.0, 3.0]);
    }

    #[test]
    fn malformed_json_is_a_parse_error_tagged_with_the_provider() {
        let err = parse_agy_response("this is not json").unwrap_err();
        assert!(matches!(err, LlmError::ParseResponse { .. }));
        assert_eq!(err.provider(), PROVIDER);
        assert!(!err.is_retryable(), "a bad body will not parse on retry");
    }

    #[test]
    fn a_missing_choices_field_is_a_parse_error() {
        let err = parse_agy_response(r#"{"unexpected":true}"#).unwrap_err();
        assert!(matches!(err, LlmError::ParseResponse { .. }));
    }

    #[test]
    fn a_choice_missing_its_message_is_a_parse_error() {
        let err = parse_agy_response(r#"{"choices":[{}]}"#).unwrap_err();
        assert!(matches!(err, LlmError::ParseResponse { .. }));
    }

    #[test]
    fn the_provider_constant_matches_what_errors_report() {
        let err = parse_agy_response("{").unwrap_err();
        assert_eq!(err.provider(), PROVIDER);
        assert_eq!(PROVIDER, "agy");
        assert_eq!(API_KEY_VAR, "AGY_API_KEY");
    }
}
