//! Bearer API keys and per-key rate limits.
//!
//! Keys are held only as SHA-256 hashes. A presented key is hashed and
//! compared with every stored hash in constant time, without stopping at
//! a match, so the comparison does not reveal how much of a key was right
//! or which stored key it was. Each key has a token bucket, and
//! `/v1/report` a second, smaller one, because every call costs provider
//! tokens.

use crate::app::AppState;
use crate::problem::Problem;
use axum::extract::{Request, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use governor::clock::{Clock, DefaultClock};
use governor::{DefaultKeyedRateLimiter, Quota, RateLimiter};
use sha2::{Digest, Sha256};
use std::num::NonZeroU32;
use std::sync::Arc;
use subtle::{Choice, ConstantTimeEq};

/// The path whose calls also count against the report limit.
pub const REPORT_PATH: &str = "/v1/report";

/// The SHA-256 hash of a key, as `NALUFX_API_KEY_SHA256` stores it.
#[must_use]
pub fn hash_key(key: &str) -> [u8; 32] {
    Sha256::digest(key.as_bytes()).into()
}

/// Whether `candidate` equals any of `hashes`, in time that depends only
/// on how many hashes there are.
#[must_use]
pub fn matches(hashes: &[[u8; 32]], candidate: &[u8; 32]) -> bool {
    hashes.iter().fold(Choice::from(0), |found, h| found | h.ct_eq(candidate)).into()
}

/// The per-key token buckets.
#[derive(Debug)]
pub struct Limits {
    general: DefaultKeyedRateLimiter<[u8; 32]>,
    report: DefaultKeyedRateLimiter<[u8; 32]>,
}

impl Limits {
    /// Buckets refilling at these rates per key per minute; a key may
    /// spend a whole minute's allowance at once.
    #[must_use]
    pub fn per_minute(general: u32, report: u32) -> Self {
        let quota = |n: u32| Quota::per_minute(NonZeroU32::new(n).unwrap_or(NonZeroU32::MIN));
        Self {
            general: RateLimiter::keyed(quota(general)),
            report: RateLimiter::keyed(quota(report)),
        }
    }

    /// `None` if the call may go ahead, or the seconds to wait.
    #[must_use]
    pub fn check(&self, key: &[u8; 32], report: bool) -> Option<u64> {
        let wait = |limiter: &DefaultKeyedRateLimiter<[u8; 32]>| {
            limiter.check_key(key).err().map(|not_until| {
                let secs = not_until.wait_time_from(DefaultClock::default().now()).as_secs_f64();
                (secs.ceil() as u64).max(1)
            })
        };
        wait(&self.general).or_else(|| if report { wait(&self.report) } else { None })
    }
}

/// Middleware: rejects a request without a valid key (401) or over its
/// key's rate (429 with `Retry-After`).
pub async fn authenticate(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Response {
    let Some(key) = bearer(&request) else {
        return unauthorized("send an API key as `Authorization: Bearer <key>`");
    };
    let hash = hash_key(key);
    if !matches(&state.config.key_hashes, &hash) {
        return unauthorized("the API key is not valid");
    }
    if let Some(seconds) = state.limits.check(&hash, request.uri().path() == REPORT_PATH) {
        let detail = format!("this key is over its rate limit; retry in {seconds} s");
        return Problem::new(StatusCode::TOO_MANY_REQUESTS, detail)
            .retry_after(seconds)
            .into_response();
    }
    next.run(request).await
}

/// The bearer token, if the request has one.
fn bearer(request: &Request) -> Option<&str> {
    let value = request.headers().get(header::AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = value.split_once(' ')?;
    (scheme.eq_ignore_ascii_case("bearer") && !token.trim().is_empty()).then(|| token.trim())
}

/// A 401 problem with the `WWW-Authenticate` challenge.
fn unauthorized(detail: &str) -> Response {
    let mut response = Problem::new(StatusCode::UNAUTHORIZED, detail).into_response();
    let _ =
        response.headers_mut().insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_hashes_to_its_sha256() {
        // printf %s test | sha256sum
        assert_eq!(
            hex::encode(hash_key("test")),
            "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
        );
    }

    #[test]
    fn a_key_matches_only_its_own_hash() {
        let stored = [hash_key("alpha"), hash_key("beta")];
        assert!(matches(&stored, &hash_key("beta")));
        assert!(!matches(&stored, &hash_key("gamma")));
        assert!(!matches(&[], &hash_key("alpha")));
    }

    #[test]
    fn a_key_over_its_rate_is_told_how_long_to_wait() {
        let limits = Limits::per_minute(2, 1);
        let key = hash_key("k");
        assert_eq!(limits.check(&key, false), None);
        assert_eq!(limits.check(&key, false), None);
        assert!(limits.check(&key, false).is_some_and(|s| (1..=60).contains(&s)));
        // Another key has its own bucket.
        assert_eq!(limits.check(&hash_key("other"), false), None);
    }

    #[test]
    fn reports_have_their_own_smaller_bucket() {
        let limits = Limits::per_minute(100, 1);
        let key = hash_key("k");
        assert_eq!(limits.check(&key, true), None);
        assert!(limits.check(&key, true).is_some());
        assert_eq!(limits.check(&key, false), None, "the general bucket still has room");
    }
}
