use crate::llm::openai::{get_openai_api_key, parse_openai_response, send_openai_request};
use crate::{
    models::cash_flow_dm::{CashFlowRequest, CashFlowResponse},
    utils::calculations::calculate_optimal_allocation,
};
use actix_web::{post, web, HttpResponse, Responder};
use log::{debug, error};
use reqwest::Client;
use serde_json::json;

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
