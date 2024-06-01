use crate::api::models::{CashFlowRequest, CashFlowResponse};
use crate::utils::calculations::calculate_optimal_allocation;
use actix_web::{post, web, HttpResponse, Responder};
use log::{debug, error};
use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::env;

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAIMessage {
    content: String,
}

#[post("/predict")]
async fn predict_cash_flow(data: web::Json<CashFlowRequest>) -> impl Responder {
    let client = Client::new();
    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            error!("OPENAI_API_KEY is not set");
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

    let response = match client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(err) => {
            error!("Error sending request to OpenAI API: {:?}", err);
            return HttpResponse::InternalServerError().body("Error contacting OpenAI API");
        }
    };

    if !response.status().is_success() {
        error!(
            "OpenAI API call failed with status: {:?}",
            response.status()
        );
        return HttpResponse::InternalServerError().body("OpenAI API call failed");
    }

    let body = match response.text().await {
        Ok(body) => body,
        Err(err) => {
            error!("Error reading response body: {:?}", err);
            return HttpResponse::InternalServerError().body("Error reading response body");
        }
    };

    let openai_response: OpenAIResponse = match serde_json::from_str(&body) {
        Ok(json) => json,
        Err(err) => {
            error!("Error parsing response JSON: {:?}", err);
            return HttpResponse::InternalServerError().body("Error parsing response JSON");
        }
    };

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

    // Ensure predictions have a length of 6
    let predictions = if predictions.len() == 6 {
        predictions
    } else {
        vec![0.0; 6]
    };

    let optimal_allocation = calculate_optimal_allocation(&predictions);

    HttpResponse::Ok().json(CashFlowResponse {
        predictions,
        optimal_allocation,
    })
}
