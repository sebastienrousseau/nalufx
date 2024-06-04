use crate::{
    api::models::{CashFlowRequest, CashFlowResponse},
    utils::calculations::calculate_optimal_allocation,
};
use actix_web::{post, web, HttpResponse, Responder};
use dotenvy::dotenv;
use log::{debug, error};
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use std::env;

#[derive(Debug, Deserialize)]
pub struct OpenAIResponse {
    /// The choices made by the model.
    pub choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIChoice {
    /// The content of the choice.
    pub message: OpenAIMessage,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIMessage {
    pub content: String,
}

/// Fetches the OpenAI API key from the environment or the .env file.
///
/// Returns the API key as a string if it exists in the environment or the .env file, or an error message if it doesn't.
pub fn get_openai_api_key() -> Result<String, &'static str> {
    // First, try to read the API key from the environment variables
    if let Ok(key) = env::var("OPENAI_API_KEY") {
        return Ok(key);
    }

    // If the API key is not found in the environment variables, try to read it from the .env file
    dotenv().ok();
    match env::var("OPENAI_API_KEY") {
        Ok(key) => Ok(key),
        Err(_) => Err("OPENAI_API_KEY is not set in the environment or the .env file"),
    }
}

/// Sends a request to the OpenAI API and handles the response.
///
/// # Arguments
///
/// * `client` - A reference to the HTTP client.
/// * `api_key` - The OpenAI API key.
/// * `request_body` - The JSON request body to send to the API.
///
/// Returns the response body as a string if the request is successful, or an error message if it fails.
pub async fn send_openai_request(
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
            error!("Error sending request to OpenAI API: {:?}", err);
            "Error contacting OpenAI API"
        })?;
    if !response.status().is_success() {
        error!(
            "OpenAI API call failed with status: {:?}",
            response.status()
        );
        return Err("OpenAI API call failed");
    }
    response.text().await.map_err(|err| {
        error!("Error reading response body: {:?}", err);
        "Error reading response body"
    })
}
// pub async fn send_openai_request(
//     client: &Client,
//     api_key: &str,
//     request_body: Value,
// ) -> Result<String, &'static str> {
//     let response = client
//         .post("https://api.openai.com/v1/chat/completions")
//         .header("Authorization", format!("Bearer {}", api_key))
//         .json(&request_body)
//         .send()
//         .await
//         .map_err(|err| {
//             error!("Error sending request to OpenAI API: {:?}", err);
//             "Error contacting OpenAI API"
//         })?;

//     if !response.status().is_success() {
//         error!(
//             "OpenAI API call failed with status: {:?}",
//             response.status()
//         );
//         return Err("OpenAI API call failed");
//     }

//     response.text().await.map_err(|err| {
//         error!("Error reading response body: {:?}", err);
//         "Error reading response body"
//     })
// }

/// Parses the OpenAI API response JSON.
///
/// # Arguments
///
/// * `body` - The response body as a string.
///
/// Returns a vector of predicted values if parsing is successful, or an `HttpResponse` with an internal server error if parsing fails.
pub fn parse_openai_response(body: &str) -> Result<Vec<f64>, HttpResponse> {
    let openai_response: OpenAIResponse = serde_json::from_str(body).map_err(|err| {
        error!("Error parsing response JSON: {:?}", err);
        HttpResponse::InternalServerError().body("Error parsing response JSON")
    })?;

    let predictions: Vec<f64> = openai_response
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

#[post("/predict")]
async fn predict_cash_flow(
    data: web::Json<CashFlowRequest>,
    daily_returns: web::Json<Vec<f64>>,
    cash_flows: web::Json<Vec<f64>>,
    market_indices: web::Json<Vec<f64>>,
    fund_characteristics: web::Json<Vec<f64>>,
) -> impl Responder {
    let client = Client::new();
    let api_key = match get_openai_api_key() {
        Ok(key) => key,
        Err(err) => {
            error!("{}", err);
            return HttpResponse::InternalServerError().body("Internal Server Error");
        }
    };

    if data.historical_data.is_empty() {
        error!("Historical data is empty");
        return HttpResponse::BadRequest().body("Invalid historical data");
    }

    let historical_data_str = data
        .historical_data
        .iter()
        .map(|d| d.to_string())
        .collect::<Vec<_>>()
        .join(", ");

    let request_body = json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            {"role": "system", "content": "You are a highly skilled financial assistant with expertise in forecasting cash flows and optimizing financial allocations to enhance returns while minimizing risks. Your predictions are based on thorough analysis of historical data and contemporary financial models."},
            {"role": "user", "content": format!("Based on the provided historical cash flow data: [{}], please predict the cash flow values for the upcoming week. Additionally, suggest an optimal allocation strategy that maximizes returns and minimizes risks. The historical data is presented in chronological order, from the earliest to the most recent.", historical_data_str)}
        ],
        "max_tokens": 100,
    });

    debug!("Request body: {:?}", request_body);

    let openai_url = "https://api.openai.com/v1/chat/completions";
    let body = match send_openai_request(&client, openai_url, &api_key, request_body).await {
        Ok(body) => body,
        Err(err) => return HttpResponse::InternalServerError().body(err),
    };

    let predictions = match parse_openai_response(&body) {
        Ok(predictions) => predictions,
        Err(err) => return err,
    };

    // Ensure predictions have a length of 6
    let predictions = if predictions.len() == 6 {
        predictions
    } else {
        vec![0.0; 6]
    };

    // Calculate the optimal allocation based on predictions
    let optimal_allocation_result = calculate_optimal_allocation(
        &daily_returns,
        &cash_flows,
        &market_indices,
        &fund_characteristics,
        predictions.len(),
    );

    match optimal_allocation_result {
        Ok(optimal_allocation) => HttpResponse::Ok().json(CashFlowResponse {
            predictions,
            optimal_allocation,
        }),
        Err(e) => {
            error!("Error calculating optimal allocation: {}", e);
            HttpResponse::InternalServerError().body("Error calculating optimal allocation")
        }
    }
}
