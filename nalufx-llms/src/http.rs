//! The HTTP client every provider shares, with timeouts and retries.

use crate::error::LlmError;
use crate::provider::ClientConfig;
use reqwest::header::{HeaderMap, RETRY_AFTER};
use serde::de::DeserializeOwned;
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hasher};
use std::time::Duration;

/// How much of an error body an [`LlmError::Status`] keeps.
const BODY_EXCERPT: usize = 500;

/// One connection pool with its timeouts and retry policy. Cloning it
/// shares the pool.
#[derive(Clone, Debug)]
pub struct Http {
    client: reqwest::Client,
    config: ClientConfig,
}

impl Http {
    /// A client with these timeouts and retries.
    ///
    /// # Errors
    ///
    /// [`LlmError::Config`] if there are no attempts, or the TLS backend
    /// cannot be initialised.
    pub fn new(config: ClientConfig) -> Result<Self, LlmError> {
        if config.max_attempts == 0 {
            return Err(LlmError::Config("max_attempts must be at least 1".to_string()));
        }
        let client = reqwest::Client::builder()
            .connect_timeout(config.connect_timeout)
            .timeout(config.request_timeout)
            .build()
            .map_err(|e| LlmError::Config(format!("the HTTP client could not be built: {e}")))?;
        Ok(Self { client, config })
    }

    /// The retry policy and timeouts.
    #[must_use]
    pub const fn config(&self) -> ClientConfig {
        self.config
    }

    /// POSTs a JSON body and parses the JSON reply, retrying a retryable
    /// failure (a timeout, a transport error, 429 or a 5xx) up to the
    /// configured attempts.
    ///
    /// # Errors
    ///
    /// The last attempt's [`LlmError`], or a [`LlmError::ParseResponse`]
    /// if a 2xx body is not the expected shape (not retried).
    pub async fn post_json<T: DeserializeOwned>(
        &self,
        provider: &'static str,
        url: &str,
        headers: &[(&str, &str)],
        body: &serde_json::Value,
    ) -> Result<T, LlmError> {
        let mut attempt = 1;
        loop {
            match self.send(provider, url, headers, body).await {
                Ok(text) => return parse(provider, &text),
                Err(e) if e.is_retryable() && attempt < self.config.max_attempts => {
                    log::warn!("{provider}: attempt {attempt} failed, retrying: {e}");
                    tokio::time::sleep(self.delay(attempt, &e)).await;
                    attempt += 1;
                },
                Err(e) => return Err(e),
            }
        }
    }

    /// One attempt: the body of a 2xx reply, or the error.
    async fn send(
        &self,
        provider: &'static str,
        url: &str,
        headers: &[(&str, &str)],
        body: &serde_json::Value,
    ) -> Result<String, LlmError> {
        let mut request = self.client.post(url).json(body);
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        let response = request.send().await.map_err(|e| transport(provider, &e))?;
        let status = response.status();
        let retry_after = retry_after(response.headers());
        let text = response.text().await.map_err(|e| transport(provider, &e))?;
        if status.is_success() {
            return Ok(text);
        }
        Err(LlmError::Status {
            provider,
            status: status.as_u16(),
            retry_after,
            body: excerpt(&text),
        })
    }

    /// The wait before attempt `attempt + 1`: the provider's `Retry-After`
    /// if it gave one, else `base * 2^(attempt - 1)` scaled by a random
    /// factor in `[0.5, 1)`; never more than the configured maximum.
    fn delay(&self, attempt: u32, error: &LlmError) -> Duration {
        let wait = error.retry_after().unwrap_or_else(|| {
            let backoff = self.config.base_delay.saturating_mul(1 << (attempt - 1).min(16));
            backoff.mul_f64(jitter())
        });
        wait.min(self.config.max_delay)
    }
}

/// A transport error, timeout or not.
fn transport(provider: &'static str, error: &reqwest::Error) -> LlmError {
    LlmError::Transport { provider, message: error.to_string(), timeout: error.is_timeout() }
}

/// Parses a 2xx body.
fn parse<T: DeserializeOwned>(provider: &'static str, text: &str) -> Result<T, LlmError> {
    serde_json::from_str(text)
        .map_err(|e| LlmError::ParseResponse { provider, message: e.to_string() })
}

/// The `Retry-After` header in seconds. The HTTP-date form is not used by
/// the supported providers and is ignored.
fn retry_after(headers: &HeaderMap) -> Option<Duration> {
    headers.get(RETRY_AFTER)?.to_str().ok()?.trim().parse::<u64>().ok().map(Duration::from_secs)
}

/// The first [`BODY_EXCERPT`] characters of a body.
fn excerpt(text: &str) -> String {
    text.chars().take(BODY_EXCERPT).collect()
}

/// A random factor in `[0.5, 1)`, from the standard library's randomly
/// seeded hasher, so retries from many clients do not move in step.
fn jitter() -> f64 {
    let bits = RandomState::new().build_hasher().finish();
    0.5 + (bits >> 11) as f64 / (1u64 << 54) as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::HeaderValue;

    fn http(max_attempts: u32) -> Http {
        Http::new(ClientConfig { max_attempts, ..ClientConfig::default() }).unwrap()
    }

    #[test]
    fn zero_attempts_is_a_configuration_error() {
        assert!(matches!(
            Http::new(ClientConfig { max_attempts: 0, ..ClientConfig::default() }),
            Err(LlmError::Config(_))
        ));
    }

    #[test]
    fn retry_after_in_seconds_is_read() {
        let mut h = HeaderMap::new();
        assert_eq!(retry_after(&h), None);
        let _ = h.insert(RETRY_AFTER, HeaderValue::from_static("7"));
        assert_eq!(retry_after(&h), Some(Duration::from_secs(7)));
        let _ = h.insert(RETRY_AFTER, HeaderValue::from_static("Wed, 21 Oct 2026 07:28:00 GMT"));
        assert_eq!(retry_after(&h), None);
    }

    #[test]
    fn the_delay_honours_retry_after_up_to_the_maximum() {
        let h = http(3);
        let asked = |secs| LlmError::Status {
            provider: "p",
            status: 429,
            retry_after: Some(Duration::from_secs(secs)),
            body: String::new(),
        };
        assert_eq!(h.delay(1, &asked(2)), Duration::from_secs(2));
        assert_eq!(h.delay(1, &asked(3600)), h.config().max_delay);
    }

    #[test]
    fn the_backoff_doubles_within_its_jitter() {
        let h = http(3);
        let e = LlmError::Transport { provider: "p", message: String::new(), timeout: true };
        for attempt in 1..=3 {
            let base = h.config().base_delay.saturating_mul(1 << (attempt - 1));
            let d = h.delay(attempt, &e);
            assert!(d >= base / 2 && d < base, "attempt {attempt}: {d:?}");
        }
    }

    #[test]
    fn jitter_stays_in_range() {
        for _ in 0..1000 {
            let j = jitter();
            assert!((0.5..1.0).contains(&j), "{j}");
        }
    }

    #[test]
    fn long_bodies_are_cut_to_an_excerpt() {
        assert_eq!(excerpt(&"x".repeat(2000)).len(), BODY_EXCERPT);
        assert_eq!(excerpt("short"), "short");
    }
}
