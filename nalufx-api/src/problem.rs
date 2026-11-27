//! RFC 9457 problem details: the body of every non-2xx response.
//!
//! Handlers return [`Problem`]s. Middleware (the body limit, the timeout,
//! axum's own extractor rejections) produces plain responses, so
//! [`normalise`] rewrites any non-2xx response that is not already
//! `application/problem+json`, keeping its text as the detail. Every
//! problem carries a stable type URI and the request id.

use axum::body::to_bytes;
use axum::extract::Request;
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// The media type of a problem details body.
pub const PROBLEM_JSON: &str = "application/problem+json";

/// The base of every problem type URI.
pub const TYPE_BASE: &str = "https://nalufx.com/problems/";

/// The request id header, set on every request and response.
pub const REQUEST_ID: &str = "x-request-id";

/// The most of a plain error body kept as a problem's detail.
const DETAIL_LIMIT: usize = 64 * 1024;

/// An RFC 9457 problem details body.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Problem {
    /// A stable URI naming the problem type, under `https://nalufx.com/problems/`.
    #[serde(rename = "type")]
    pub problem_type: String,
    /// The status code's reason phrase.
    pub title: String,
    /// The HTTP status code.
    pub status: u16,
    /// What went wrong with this request.
    pub detail: String,
    /// The request's `x-request-id`, for tracing it through the logs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Seconds to wait before retrying, for 429 and 503.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_after: Option<u64>,
}

impl Problem {
    /// A problem with this status and detail; the type and title follow
    /// from the status.
    #[must_use]
    pub fn new(status: StatusCode, detail: impl Into<String>) -> Self {
        Self {
            problem_type: format!("{TYPE_BASE}{}", slug(status)),
            title: status.canonical_reason().unwrap_or("Error").to_string(),
            status: status.as_u16(),
            detail: detail.into(),
            request_id: None,
            retry_after: None,
        }
    }

    /// The same problem, telling the caller to retry after `seconds`.
    #[must_use]
    pub const fn retry_after(mut self, seconds: u64) -> Self {
        self.retry_after = Some(seconds);
        self
    }

    /// The HTTP status.
    #[must_use]
    pub fn status_code(&self) -> StatusCode {
        StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl IntoResponse for Problem {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let retry = self.retry_after;
        let body = serde_json::to_vec(&self).unwrap_or_default();
        let mut response = (status, body).into_response();
        let headers = response.headers_mut();
        let _ = headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(PROBLEM_JSON));
        if let Some(seconds) = retry.and_then(|s| HeaderValue::from_str(&s.to_string()).ok()) {
            let _ = headers.insert(header::RETRY_AFTER, seconds);
        }
        response
    }
}

/// The slug of each status with its own problem type.
const SLUGS: [(u16, &str); 13] = [
    (400, "invalid-request"),
    (401, "unauthorized"),
    (404, "not-found"),
    (405, "method-not-allowed"),
    (408, "timeout"),
    (410, "gone"),
    (413, "payload-too-large"),
    (415, "unsupported-media-type"),
    (422, "unprocessable"),
    (429, "rate-limited"),
    (502, "bad-gateway"),
    (503, "unavailable"),
    (504, "gateway-timeout"),
];

/// The stable slug for a status: the last segment of the problem type URI.
/// A status without its own type is `internal` (5xx) or `client-error`.
#[must_use]
pub fn slug(status: StatusCode) -> &'static str {
    let code = status.as_u16();
    SLUGS
        .iter()
        .find(|(s, _)| *s == code)
        .map_or(if code >= 500 { "internal" } else { "client-error" }, |(_, slug)| slug)
}

/// Middleware: makes every non-2xx response a problem with the request
/// id, and sets the id on every response.
pub async fn normalise(request: Request, next: Next) -> Response {
    let id = request.headers().get(REQUEST_ID).cloned();
    let response = next.run(request).await;
    let mut response = if response.status().is_success() || response.status().is_informational() {
        response
    } else {
        as_problem(response, id.as_ref().and_then(|v| v.to_str().ok())).await
    };
    if let Some(id) = id {
        let _ = response.headers_mut().insert(REQUEST_ID, id);
    }
    response
}

/// An error response as a problem with the request id: a handler's
/// problem gets the id added; any other body becomes the detail.
async fn as_problem(response: Response, request_id: Option<&str>) -> Response {
    let (parts, body) = response.into_parts();
    let bytes = to_bytes(body, DETAIL_LIMIT).await.unwrap_or_default();
    let is_problem = parts.headers.get(header::CONTENT_TYPE).is_some_and(|v| v == PROBLEM_JSON);
    let mut problem = if is_problem {
        serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| Problem::new(parts.status, "malformed problem"))
    } else {
        Problem::new(parts.status, detail_from(&bytes, parts.status))
    };
    problem.request_id = request_id.map(str::to_string);
    let mut out = problem.into_response();
    for name in [header::ALLOW, header::WWW_AUTHENTICATE, header::RETRY_AFTER, header::LINK] {
        if let Some(value) = parts.headers.get(&name) {
            let _ = out.headers_mut().insert(name, value.clone());
        }
    }
    *out.status_mut() = parts.status;
    out
}

/// A plain body as a detail, or the reason phrase if it is empty.
fn detail_from(bytes: &[u8], status: StatusCode) -> String {
    let text = String::from_utf8_lossy(bytes).trim().to_string();
    if text.is_empty() {
        status.canonical_reason().unwrap_or("Error").to_string()
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_status_has_a_stable_type() {
        assert_eq!(
            Problem::new(StatusCode::TOO_MANY_REQUESTS, "slow down").problem_type,
            "https://nalufx.com/problems/rate-limited"
        );
        assert_eq!(slug(StatusCode::IM_A_TEAPOT), "client-error");
        assert_eq!(slug(StatusCode::NOT_IMPLEMENTED), "internal");
    }

    #[test]
    fn a_problem_response_is_problem_json_with_retry_after() {
        let r = Problem::new(StatusCode::TOO_MANY_REQUESTS, "x").retry_after(3).into_response();
        assert_eq!(r.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(r.headers()[header::CONTENT_TYPE], PROBLEM_JSON);
        assert_eq!(r.headers()[header::RETRY_AFTER], "3");
    }

    #[test]
    fn an_empty_body_becomes_the_reason_phrase() {
        assert_eq!(detail_from(b"  ", StatusCode::PAYLOAD_TOO_LARGE), "Payload Too Large");
        assert_eq!(detail_from(b"too big", StatusCode::PAYLOAD_TOO_LARGE), "too big");
    }
}
