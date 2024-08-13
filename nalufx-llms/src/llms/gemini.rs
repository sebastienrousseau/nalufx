use crate::models::gemini_dm::GeminiResponse;
use actix_web::HttpResponse;
use dotenvy::dotenv;
use log::error;
use reqwest::Client;
use std::env;

/// Retrieves the Gemini API key from the environment variables or.env file.
///
/// # Returns
///
/// * `Ok(String)` - If the API key is successfully retrieved.
/// * `Err(&'static str)` - If the API key is not found in the environment variables or.env file.
pub fn get_gemini_api_key() -> Result<String, &'static str> {
    // First, try to read the API key from the environment variables
    if let Ok(key) = env::var("GEMINI_API_KEY") {
        return Ok(key);
    }

    // If the API key is not found in the environment variables, try to read it from the .env file
    match dotenv() {
        Ok(_) => match env::var("GEMINI_API_KEY") {
            Ok(key) => Ok(key),
            Err(_) => {
                error!("GEMINI_API_KEY not found in the .env file");
                Err("GEMINI_API_KEY not found in the .env file")
            },
        },
        Err(err) => {
            error!("Failed to load .env file: {:?}", err);
            Err("Failed to load .env file")
        },
    }
}

/// Sends a POST request to the Gemini API with the provided request body.
///
/// # Arguments
///
/// * `client` - A reference to the reqwest::Client instance used for making HTTP requests.
/// * `api_url` - A string representing the URL of the Gemini API endpoint.
/// * `api_key` - A string representing the API key for authentication.
/// * `request_body` - A serde_json::Value representing the JSON payload to be sent in the request body.
///
/// # Returns
///
/// * `Ok(String)` - If the request is successfully sent and the response body is returned as a string.
/// * `Err(&'static str)` - If an error occurs during the request or response handling.
pub async fn send_gemini_request(
    client: &Client,
    api_url: &str,
    api_key: &str,
    request_body: serde_json::Value,
) -> Result<serde_json::Value, reqwest::Error> {
    let response = client.post(api_url).bearer_auth(api_key).json(&request_body).send().await?;

    let json_response: serde_json::Value = response.json().await?;
    Ok(json_response)
}

/// Parses the Gemini API response and extracts the predictions.
///
/// # Arguments
///
/// * `body` - A string representing the JSON response from the Gemini API.
///
/// # Returns
///
/// * `Ok(Vec<f64>)` - If the response is successfully parsed and the predictions are extracted.
/// * `Err(actix_web::HttpResponse)` - If an error occurs during parsing or if the response is invalid.
///
/// # Errors
///
/// * If the JSON response cannot be parsed into the `GeminiResponse` struct, an error is returned with an
///   InternalServerError status and a message indicating the parsing error.
/// * If any of the prediction values cannot be parsed into a `f64`, the `unwrap_or_default` method is used
///   to provide a default value of `0.0`.
pub fn parse_gemini_response(body: &str) -> Result<Vec<f64>, HttpResponse> {
    let gemini_response: GeminiResponse = serde_json::from_str(body).map_err(|err| {
        error!("Error parsing response JSON: {:?}", err);
        HttpResponse::InternalServerError().body("Error parsing response JSON")
    })?;

    let predictions: Vec<f64> = gemini_response
        .choices
        .iter()
        .flat_map(|choice| {
            choice.message.content.split_whitespace().map(|s| s.parse().unwrap_or_default())
        })
        .collect();

    Ok(predictions)
}
