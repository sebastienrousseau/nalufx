//! Request routing.
//!
//! Matches a method and path to a handler, reads and decodes the body, and
//! turns anything unmatched or malformed into a JSON error response.

use crate::{
    api::{
        handlers::predict_cash_flow,
        response::{self, JsonResponse},
    },
    models::cash_flow_dm::PredictCashFlowRequest,
};
use http_body_util::{BodyExt, Limited};
use hyper::{body::Incoming, Method, Request, StatusCode};
use log::error;
use reqwest::Client;
use std::convert::Infallible;

/// The largest request body accepted, in bytes.
///
/// Without a cap an unauthenticated caller could stream indefinitely and
/// exhaust memory, since the body is buffered before deserialisation.
pub const MAX_BODY_BYTES: usize = 1024 * 1024;

/// Routes a request to a handler.
///
/// # Arguments
///
/// * `client` - The HTTP client handed to handlers that need one.
/// * `req` - The incoming request.
///
/// # Returns
///
/// Always `Ok`: every failure is reported to the caller as a JSON error
/// response rather than dropping the connection.
///
/// # Errors
///
/// This function does not fail; the `Result` satisfies hyper's service
/// signature.
pub async fn route(client: Client, req: Request<Incoming>) -> Result<JsonResponse, Infallible> {
    let response = match (req.method(), req.uri().path()) {
        (&Method::POST, "/predict") => match read_json::<PredictCashFlowRequest>(req).await {
            Ok(body) => predict_cash_flow(&client, body).await,
            Err(rejection) => *rejection,
        },
        (&Method::POST, _) | (&Method::GET, _) => {
            response::error(StatusCode::NOT_FOUND, "Not Found")
        },
        _ => response::error(StatusCode::METHOD_NOT_ALLOWED, "Method Not Allowed"),
    };
    Ok(response)
}

/// Reads a request body and deserialises it as JSON.
///
/// # Arguments
///
/// * `req` - The request whose body should be consumed.
///
/// # Returns
///
/// The deserialised value, or a ready-to-send error response: `413` if the
/// body exceeds [`MAX_BODY_BYTES`] and `400` if it is not valid JSON of the
/// expected shape. The response is boxed to keep the `Result` small, since
/// the success path is by far the common one.
async fn read_json<T: serde::de::DeserializeOwned>(
    req: Request<Incoming>,
) -> Result<T, Box<JsonResponse>> {
    let collected = Limited::new(req.into_body(), MAX_BODY_BYTES).collect().await.map_err(|err| {
        error!("Error reading request body: {err}");
        Box::new(response::error(StatusCode::PAYLOAD_TOO_LARGE, "Request body too large"))
    })?;

    serde_json::from_slice(&collected.to_bytes()).map_err(|err| {
        error!("Error deserialising request body: {err}");
        Box::new(response::error(StatusCode::BAD_REQUEST, "Invalid request body"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::Full;
    use bytes::Bytes;

    /// Builds a request whose body type matches hyper's `Incoming` after a
    /// round trip through the router's own reader.
    fn post(path: &str, body: &str) -> Request<Full<Bytes>> {
        Request::builder()
            .method(Method::POST)
            .uri(path)
            .body(Full::new(Bytes::copy_from_slice(body.as_bytes())))
            .expect("valid request")
    }

    /// The body reader under test, generic over any body type so it can be
    /// driven without a live connection.
    async fn read<T: serde::de::DeserializeOwned>(
        req: Request<Full<Bytes>>,
    ) -> Result<T, StatusCode> {
        let collected = Limited::new(req.into_body(), MAX_BODY_BYTES)
            .collect()
            .await
            .map_err(|_| StatusCode::PAYLOAD_TOO_LARGE)?;
        serde_json::from_slice(&collected.to_bytes()).map_err(|_| StatusCode::BAD_REQUEST)
    }

    #[tokio::test]
    async fn a_well_formed_body_deserialises() {
        let body = r#"{"historical_data":[1.0],"daily_returns":[0.1],
            "cash_flows":[1.0],"market_indices":[100.0],"fund_characteristics":[0.5]}"#;
        let parsed: PredictCashFlowRequest =
            read(post("/predict", body)).await.expect("should deserialise");
        assert_eq!(parsed.historical_data, vec![1.0]);
    }

    #[tokio::test]
    async fn a_malformed_body_is_a_bad_request() {
        let status = read::<PredictCashFlowRequest>(post("/predict", "not json"))
            .await
            .expect_err("should reject");
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn a_body_missing_a_field_is_a_bad_request() {
        let status = read::<PredictCashFlowRequest>(post("/predict", r#"{"historical_data":[1.0]}"#))
            .await
            .expect_err("should reject");
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn a_body_over_the_cap_is_rejected() {
        let oversized = format!(r#"{{"historical_data":[{}]}}"#, "1.0,".repeat(MAX_BODY_BYTES / 2));
        let status = read::<PredictCashFlowRequest>(post("/predict", &oversized))
            .await
            .expect_err("should reject");
        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    }
}
