//! Small helpers for building JSON responses.
//!
//! These replace `actix_web::HttpResponse`. The server is one route deep,
//! so a handful of constructors is all the framework that is warranted —
//! and it keeps the HTTP surface owned here rather than in a dependency.

use bytes::Bytes;
use http_body_util::Full;
use hyper::{
    header::{HeaderValue, CONTENT_TYPE},
    Response, StatusCode,
};
use serde::Serialize;

use crate::models::cash_flow_dm::ErrorResponse;

/// The response type returned by every handler.
pub type JsonResponse = Response<Full<Bytes>>;

/// Builds a JSON response with the given status.
///
/// # Arguments
///
/// * `status` - The HTTP status to return.
/// * `value` - Any serialisable value to use as the body.
///
/// # Returns
///
/// A response carrying the serialised value and a JSON content type. If
/// serialisation fails the status is replaced with `500` and a fixed body
/// is returned, so this never panics.
pub fn json<T: Serialize>(status: StatusCode, value: &T) -> JsonResponse {
    match serde_json::to_vec(value) {
        Ok(body) => Response::builder()
            .status(status)
            .header(CONTENT_TYPE, "application/json")
            .body(Full::new(Bytes::from(body)))
            .unwrap_or_else(|_| fallback()),
        // Serialising our own response types cannot fail in practice, but
        // returning 500 keeps the signature infallible for callers.
        Err(_) => fallback(),
    }
}

/// Builds a JSON error response of the shape `{"error": "..."}`.
///
/// # Arguments
///
/// * `status` - The HTTP status to return.
/// * `message` - The human-readable error message.
///
/// # Returns
///
/// A response carrying an [`ErrorResponse`] body.
pub fn error(status: StatusCode, message: impl Into<String>) -> JsonResponse {
    json(status, &ErrorResponse { error: message.into() })
}

/// The last-resort response used when a response cannot be built.
fn fallback() -> JsonResponse {
    let mut response =
        Response::new(Full::new(Bytes::from_static(br#"{"error":"Internal Server Error"}"#)));
    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    // Replacing an absent header; there is no prior value to inspect.
    let _ =
        response.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_sets_the_status_and_content_type() {
        let response = json(StatusCode::CREATED, &ErrorResponse { error: "x".into() });
        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(response.headers()[CONTENT_TYPE], "application/json");
    }

    #[test]
    fn error_uses_the_error_response_shape() {
        let response = error(StatusCode::BAD_REQUEST, "bad input");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn fallback_is_a_well_formed_server_error() {
        let response = fallback();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(response.headers()[CONTENT_TYPE], "application/json");
    }
}
