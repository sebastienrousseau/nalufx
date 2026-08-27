//! Request handlers.
//!
//! Handlers take an already-deserialised request and return a response.
//! Reading and decoding the body is the router's job, which keeps these
//! functions callable from a test without standing up a socket.

use crate::{
    api::response::{self, JsonResponse},
    models::cash_flow_dm::{CashFlowResponse, PredictCashFlowRequest},
    utils::calculations::calculate_optimal_allocation,
};
use hyper::StatusCode;
use log::{debug, error};
use nalufx_llms::llms::openai::{get_openai_api_key, parse_openai_response, send_openai_request};
use reqwest::Client;
use serde_json::json;

/// The number of predicted values the model is asked for.
const PREDICTION_WINDOW: usize = 6;

/// Predicts upcoming cash flows and derives an optimal allocation.
///
/// # Arguments
///
/// * `client` - The HTTP client used to reach the model provider.
/// * `request` - The deserialised request body.
///
/// # Returns
///
/// `200` with a [`CashFlowResponse`] on success. `400` if the historical
/// data is empty, `503` if the provider is unreachable, and `500` for a
/// malformed provider response or a failed allocation.
pub async fn predict_cash_flow(client: &Client, request: PredictCashFlowRequest) -> JsonResponse {
    if request.historical_data.is_empty() {
        error!("Historical data is empty");
        return response::error(StatusCode::BAD_REQUEST, "Invalid historical data");
    }

    let api_key = match get_openai_api_key() {
        Ok(key) => key,
        Err(err) => {
            error!("{err}");
            return response::error(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error");
        },
    };

    let historical_data_str =
        request.historical_data.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(", ");

    let request_body = json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            {"role": "system", "content": "You are a highly skilled financial assistant with expertise in forecasting cash flows and optimizing financial allocations to enhance returns while minimizing risks. Your predictions are based on thorough analysis of historical data and contemporary financial models."},
            {"role": "user", "content": format!("Based on the provided historical cash flow data: [{}], please predict the cash flow values for the upcoming week. Additionally, suggest an optimal allocation strategy that maximizes returns and minimizes risks. The historical data is presented in chronological order, from the earliest to the most recent.", historical_data_str)}
        ],
        "max_tokens": 100,
    });

    debug!("Request body: {request_body:?}");

    let openai_url = "https://api.openai.com/v1/chat/completions";
    let body = match send_openai_request(client, openai_url, &api_key, request_body).await {
        Ok(body) => body,
        Err(err) => {
            error!("{err}");
            // A transport failure may succeed on retry; a malformed request
            // will not. Signalling the two apart lets callers back off.
            return if err.is_retryable() {
                response::error(
                    StatusCode::SERVICE_UNAVAILABLE,
                    format!("Upstream {} is unavailable", err.provider()),
                )
            } else {
                response::error(StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error")
            };
        },
    };

    let predictions = match parse_openai_response(&body) {
        Ok(predictions) => predictions,
        // The adapters return a transport-agnostic `LlmError`; deciding the
        // HTTP status is this layer's job, not the library's.
        Err(err) => {
            error!("{err}");
            return response::error(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error parsing response from {}", err.provider()),
            );
        },
    };

    // The allocation step is sized by the prediction window, so a short or
    // long completion is replaced rather than allowed to resize it.
    let predictions = if predictions.len() == PREDICTION_WINDOW {
        predictions
    } else {
        vec![0.0; PREDICTION_WINDOW]
    };

    match calculate_optimal_allocation(
        &request.daily_returns,
        &request.cash_flows,
        &request.market_indices,
        &request.fund_characteristics,
        predictions.len(),
    ) {
        Ok(optimal_allocation) => {
            response::json(StatusCode::OK, &CashFlowResponse { predictions, optimal_allocation })
        },
        Err(e) => {
            error!("Error calculating optimal allocation: {e}");
            response::error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Error calculating optimal allocation",
            )
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(historical_data: Vec<f64>) -> PredictCashFlowRequest {
        PredictCashFlowRequest {
            historical_data,
            daily_returns: vec![0.1, 0.2, 0.3],
            cash_flows: vec![1.0, 2.0, 3.0],
            market_indices: vec![100.0, 101.0, 102.0],
            fund_characteristics: vec![0.5, 0.6, 0.7],
        }
    }

    #[tokio::test]
    async fn empty_historical_data_is_rejected_before_any_network_call() {
        let response = predict_cash_flow(&Client::new(), request(vec![])).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
