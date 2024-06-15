use crate::models::claude_dm::ClaudeResponse;
use actix_web::HttpResponse;
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
/// * `Err(&'static str)` - If the API key is not found in the environment variables or .env file.
pub fn get_claude_api_key() -> Result<String, &'static str> {
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
                Err("CLAUDE_API_KEY not found in the .env file")
            }
        },
        Err(err) => {
            error!("Failed to load .env file: {:?}", err);
            Err("Failed to load .env file")
        }
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
/// * `Err(&'static str)` - If an error occurs during the request or response handling.
pub async fn send_claude_request(
    client: &Client,
    api_url: &str,
    api_key: &str,
    request_body: Value,
) -> Result<String, &'static str> {
    let response = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await
        .map_err(|err| {
            error!("Error sending request to Claude API: {:?}", err);
            "Error contacting Claude API"
        })?;
    if !response.status().is_success() {
        error!(
            "Claude API call failed with status: {:?}",
            response.status()
        );
        return Err("Claude API call failed");
    }
    response.text().await.map_err(|err| {
        error!("Error reading response body: {:?}", err);
        "Error reading response body"
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
/// * `Err(actix_web::HttpResponse)` - If an error occurs during parsing or if the response is invalid.
///
/// # Errors
///
/// * If the JSON response cannot be parsed into the `ClaudeResponse` struct, an error is returned with an
///   InternalServerError status and a message indicating the parsing error.
/// * If any of the prediction values cannot be parsed into a `f64`, the `unwrap_or_default` method is used
///   to provide a default value of `0.0`.
pub fn parse_claude_response(body: &str) -> Result<Vec<f64>, HttpResponse> {
    let claude_response: ClaudeResponse = serde_json::from_str(body).map_err(|err| {
        error!("Error parsing response JSON: {:?}", err);
        HttpResponse::InternalServerError().body("Error parsing response JSON")
    })?;

    let predictions: Vec<f64> = claude_response
        .choices
        .iter()
        .flat_map(|choice| {
            choice
                .message
                .content
                .split_whitespace()
                .map(|s| s.parse().unwrap_or_default())
        })
        .collect();

    Ok(predictions)
}
