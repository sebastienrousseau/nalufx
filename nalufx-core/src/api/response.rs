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

use crate::models::cash_flow_dm::ProblemDetails;

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

/// The media type of an RFC 9457 problem details body.
pub const PROBLEM_JSON: &str = "application/problem+json";

/// Builds an RFC 9457 problem details response.
///
/// The problem type is `about:blank`, so the title is the status code's
/// reason phrase and `detail` says what went wrong with this request.
///
/// # Arguments
///
/// * `status` - The HTTP status to return.
/// * `detail` - The human-readable explanation.
///
/// # Returns
///
/// A response carrying a [`ProblemDetails`] body as `application/problem+json`.
pub fn problem(status: StatusCode, detail: impl Into<String>) -> JsonResponse {
    let body = ProblemDetails {
        problem_type: "about:blank".to_string(),
        title: status.canonical_reason().unwrap_or("Unknown Status").to_string(),
        status: status.as_u16(),
        detail: detail.into(),
    };
    let mut response = json(status, &body);
    // Replacing the JSON content type `json` set; its value is not needed.
    let _ = response.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static(PROBLEM_JSON));
    response
}

/// The last-resort response used when a response cannot be built.
fn fallback() -> JsonResponse {
    let mut response = Response::new(Full::new(Bytes::from_static(
        br#"{"type":"about:blank","title":"Internal Server Error","status":500,"detail":"The response could not be built"}"#,
    )));
    *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
    // Replacing an absent header; there is no prior value to inspect.
    let _ = response.headers_mut().insert(CONTENT_TYPE, HeaderValue::from_static(PROBLEM_JSON));
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    use http_body_util::BodyExt;

    async fn body_of(response: JsonResponse) -> ProblemDetails {
        let bytes = response.into_body().collect().await.expect("body").to_bytes();
        serde_json::from_slice(&bytes).expect("a problem details body")
    }

    #[test]
    fn json_sets_the_status_and_content_type() {
        let response = json(StatusCode::CREATED, &[1, 2]);
        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(response.headers()[CONTENT_TYPE], "application/json");
    }

    #[tokio::test]
    async fn problem_is_rfc_9457_problem_json() {
        let response = problem(StatusCode::BAD_REQUEST, "bad input");
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(response.headers()[CONTENT_TYPE], PROBLEM_JSON);
        let body = body_of(response).await;
        assert_eq!(body.problem_type, "about:blank");
        assert_eq!(body.title, "Bad Request");
        assert_eq!(body.status, 400);
        assert_eq!(body.detail, "bad input");
    }

    #[tokio::test]
    async fn fallback_is_a_well_formed_server_error() {
        let response = fallback();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(response.headers()[CONTENT_TYPE], PROBLEM_JSON);
        assert_eq!(body_of(response).await.status, 500);
    }
}
