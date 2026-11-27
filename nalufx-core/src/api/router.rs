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
use bytes::Bytes;
use http_body_util::{BodyExt, Limited};
use hyper::{body::Body, Method, Request, StatusCode};
use log::error;
use std::{convert::Infallible, error::Error};

/// The largest request body accepted, in bytes.
///
/// Without a cap an unauthenticated caller could stream indefinitely and
/// exhaust memory, since the body is buffered before deserialisation.
pub const MAX_BODY_BYTES: usize = 1024 * 1024;

/// Routes a request to a handler.
///
/// # Arguments
///
/// * `req` - The incoming request, with any body type (hyper's `Incoming`
///   on a live connection).
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
pub async fn route<B>(req: Request<B>) -> Result<JsonResponse, Infallible>
where
    B: Body<Data = Bytes>,
    B::Error: Error + Send + Sync + 'static,
{
    let response = match (req.method(), req.uri().path()) {
        (&Method::POST, "/predict") => match read_json::<PredictCashFlowRequest, B>(req).await {
            Ok(body) => predict_cash_flow(&body),
            Err(rejection) => *rejection,
        },
        (&Method::POST | &Method::GET, path) => {
            response::problem(StatusCode::NOT_FOUND, format!("No route for {path}"))
        },
        (method, _) => {
            response::problem(StatusCode::METHOD_NOT_ALLOWED, format!("{method} is not supported"))
        },
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
async fn read_json<T, B>(req: Request<B>) -> Result<T, Box<JsonResponse>>
where
    T: serde::de::DeserializeOwned,
    B: Body<Data = Bytes>,
    B::Error: Error + Send + Sync + 'static,
{
    let collected =
        Limited::new(req.into_body(), MAX_BODY_BYTES).collect().await.map_err(|err| {
            error!("Error reading request body: {err}");
            let detail = format!("The request body is over {MAX_BODY_BYTES} bytes");
            Box::new(response::problem(StatusCode::PAYLOAD_TOO_LARGE, detail))
        })?;

    serde_json::from_slice(&collected.to_bytes()).map_err(|err| {
        error!("Error deserialising request body: {err}");
        Box::new(response::problem(StatusCode::BAD_REQUEST, format!("Invalid request body: {err}")))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::Full;

    fn post(path: &str, body: &str) -> Request<Full<Bytes>> {
        Request::builder()
            .method(Method::POST)
            .uri(path)
            .body(Full::new(Bytes::copy_from_slice(body.as_bytes())))
            .expect("valid request")
    }

    /// The production reader, driven with an in-memory body.
    async fn read(req: Request<Full<Bytes>>) -> Result<PredictCashFlowRequest, StatusCode> {
        read_json::<PredictCashFlowRequest, _>(req).await.map_err(|rejection| rejection.status())
    }

    #[tokio::test]
    async fn a_well_formed_body_deserialises() {
        let body = r#"{"historical_data":[1.0],"daily_returns":[0.1]}"#;
        let parsed = read(post("/predict", body)).await.expect("should deserialise");
        assert_eq!(parsed.historical_data, vec![1.0]);
        assert_eq!(parsed.daily_returns, vec![0.1]);
    }

    #[tokio::test]
    async fn a_malformed_body_is_a_bad_request() {
        let status = read(post("/predict", "not json")).await.expect_err("should reject");
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn a_body_missing_a_field_is_a_bad_request() {
        let status = read(post("/predict", r#"{"historical_data":[1.0]}"#))
            .await
            .expect_err("should reject");
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn a_body_over_the_cap_is_rejected() {
        let oversized = format!(r#"{{"historical_data":[{}]}}"#, "1.0,".repeat(MAX_BODY_BYTES / 2));
        let status = read(post("/predict", &oversized)).await.expect_err("should reject");
        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn an_unknown_path_is_not_found() {
        let response = route(post("/nope", "{}")).await.expect("infallible");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn an_unsupported_method_is_rejected() {
        let req = Request::builder()
            .method(Method::DELETE)
            .uri("/predict")
            .body(Full::new(Bytes::new()))
            .expect("valid request");
        let response = route(req).await.expect("infallible");
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }
}
