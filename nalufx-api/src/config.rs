//! Server configuration, from the environment.
//!
//! | Variable | Default | |
//! | --- | --- | --- |
//! | `NALUFX_ADDR` | `127.0.0.1:8080` | Listen address. |
//! | `NALUFX_API_KEY_SHA256` | (required) | Comma-separated hex SHA-256 hashes of the accepted API keys. |
//! | `NALUFX_CORS_ORIGINS` | none | Comma-separated origins allowed by CORS; none means CORS is denied. |
//! | `NALUFX_RATE_PER_MINUTE` | `600` | Requests per key per minute. |
//! | `NALUFX_REPORT_RATE_PER_MINUTE` | `30` | `/v1/report` calls per key per minute. |
//! | `NALUFX_TIMEOUT_SECS` | `30` | Time a request may take. |
//! | `NALUFX_CONCURRENCY` | `64` | Requests handled at once; the rest wait. |
//! | `NALUFX_LLM_PROVIDER`, `NALUFX_LLM_MODEL` | unset | The provider and model for `/v1/report`; without them it answers 503. |
//! | `NALUFX_LLM_MAX_TOKENS` | `2048` | The report's token limit. |
//!
//! Keys are stored only as hashes. Hash one with
//! `printf %s "$KEY" | sha256sum`.

use std::net::SocketAddr;
use std::time::Duration;

/// The largest request body accepted, in bytes.
pub const MAX_BODY_BYTES: usize = 1024 * 1024;

/// How long in-flight requests may drain after a shutdown signal.
pub const DRAIN: Duration = Duration::from_secs(25);

/// The language model behind `/v1/report`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LlmSettings {
    /// A provider name, as `nalufx_llms::registry::from_env` takes it.
    pub provider: String,
    /// The model's name at the provider.
    pub model: String,
    /// The report's token limit.
    pub max_tokens: u32,
}

/// How the server runs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiConfig {
    /// The listen address.
    pub addr: SocketAddr,
    /// SHA-256 hashes of the accepted API keys.
    pub key_hashes: Vec<[u8; 32]>,
    /// Origins CORS allows; empty denies all cross-origin requests.
    pub cors_origins: Vec<String>,
    /// Requests per key per minute.
    pub rate_per_minute: u32,
    /// `/v1/report` calls per key per minute.
    pub report_rate_per_minute: u32,
    /// The time a request may take before it answers 408.
    pub timeout: Duration,
    /// Requests handled at once.
    pub concurrency: usize,
    /// The provider for `/v1/report`, if any.
    pub llm: Option<LlmSettings>,
}

impl ApiConfig {
    /// A configuration for tests and embedding: these keys, the defaults
    /// otherwise, and no language model.
    #[must_use]
    pub fn with_keys(key_hashes: Vec<[u8; 32]>) -> Self {
        Self {
            addr: SocketAddr::from(([127, 0, 0, 1], 8080)),
            key_hashes,
            cors_origins: Vec::new(),
            rate_per_minute: 600,
            report_rate_per_minute: 30,
            timeout: Duration::from_secs(30),
            concurrency: 64,
            llm: None,
        }
    }

    /// Reads the configuration from the environment.
    ///
    /// # Errors
    ///
    /// A description of the first unusable variable, or of a missing
    /// `NALUFX_API_KEY_SHA256`: the server never starts without keys.
    pub fn from_env() -> Result<Self, String> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    /// Reads the configuration through `get`, which returns a variable's
    /// value if it is set.
    ///
    /// # Errors
    ///
    /// As [`ApiConfig::from_env`].
    pub fn from_lookup(get: impl Fn(&str) -> Option<String>) -> Result<Self, String> {
        let get = |name: &str| get(name).map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
        let keys = get("NALUFX_API_KEY_SHA256")
            .ok_or("NALUFX_API_KEY_SHA256 is not set: the server needs at least one API key")?;
        let mut config = Self::with_keys(parse_hashes(&keys)?);
        config.read_network(&get)?;
        config.read_limits(&get)?;
        config.llm = llm(&get)?;
        Ok(config)
    }

    /// The listen address and CORS origins.
    fn read_network(&mut self, get: &impl Fn(&str) -> Option<String>) -> Result<(), String> {
        if let Some(addr) = get("NALUFX_ADDR") {
            self.addr = addr.parse().map_err(|e| format!("NALUFX_ADDR `{addr}`: {e}"))?;
        }
        self.cors_origins = get("NALUFX_CORS_ORIGINS").map(|o| list(&o)).unwrap_or_default();
        Ok(())
    }

    /// The rate, timeout and concurrency settings.
    fn read_limits(&mut self, get: &impl Fn(&str) -> Option<String>) -> Result<(), String> {
        self.rate_per_minute = number(get, "NALUFX_RATE_PER_MINUTE", self.rate_per_minute)?;
        self.report_rate_per_minute =
            number(get, "NALUFX_REPORT_RATE_PER_MINUTE", self.report_rate_per_minute)?;
        self.timeout =
            Duration::from_secs(number(get, "NALUFX_TIMEOUT_SECS", self.timeout.as_secs())?);
        self.concurrency = number(get, "NALUFX_CONCURRENCY", self.concurrency)?;
        Ok(())
    }
}

/// The language model settings, if a provider and model are both set.
fn llm(get: &impl Fn(&str) -> Option<String>) -> Result<Option<LlmSettings>, String> {
    match (get("NALUFX_LLM_PROVIDER"), get("NALUFX_LLM_MODEL")) {
        (Some(provider), Some(model)) => {
            let max_tokens = number(get, "NALUFX_LLM_MAX_TOKENS", 2048)?;
            Ok(Some(LlmSettings { provider, model, max_tokens }))
        },
        (None, None) => Ok(None),
        _ => Err("set both NALUFX_LLM_PROVIDER and NALUFX_LLM_MODEL, or neither".to_string()),
    }
}

/// A positive number, or `default` if unset.
fn number<T: std::str::FromStr + PartialOrd + Default>(
    get: &impl Fn(&str) -> Option<String>,
    name: &str,
    default: T,
) -> Result<T, String> {
    match get(name) {
        None => Ok(default),
        Some(v) => match v.parse::<T>() {
            Ok(n) if n > T::default() => Ok(n),
            _ => Err(format!("{name} must be a positive whole number, not `{v}`")),
        },
    }
}

/// Comma-separated values, trimmed, empty ones dropped.
fn list(value: &str) -> Vec<String> {
    value.split(',').map(str::trim).filter(|v| !v.is_empty()).map(str::to_string).collect()
}

/// Comma-separated hex SHA-256 hashes.
fn parse_hashes(value: &str) -> Result<Vec<[u8; 32]>, String> {
    let hashes = list(value)
        .iter()
        .map(|h| {
            let bytes = hex::decode(h).map_err(|_| format!("`{h}` is not hex"))?;
            <[u8; 32]>::try_from(bytes)
                .map_err(|_| format!("`{h}` is not a 64-character SHA-256 hash"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if hashes.is_empty() {
        return Err("NALUFX_API_KEY_SHA256 lists no hashes".to_string());
    }
    Ok(hashes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    const HASH: &str = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";

    fn from(vars: &[(&str, &str)]) -> Result<ApiConfig, String> {
        let map: HashMap<String, String> =
            vars.iter().map(|(k, v)| ((*k).to_string(), (*v).to_string())).collect();
        ApiConfig::from_lookup(|name| map.get(name).cloned())
    }

    #[test]
    fn keys_are_required() {
        assert!(from(&[]).unwrap_err().contains("NALUFX_API_KEY_SHA256"));
        assert!(from(&[("NALUFX_API_KEY_SHA256", " , ")]).is_err());
    }

    #[test]
    fn the_defaults_are_the_planned_ones() {
        let c = from(&[("NALUFX_API_KEY_SHA256", HASH)]).unwrap();
        assert_eq!(c.key_hashes.len(), 1);
        assert_eq!((c.rate_per_minute, c.report_rate_per_minute, c.concurrency), (600, 30, 64));
        assert_eq!(c.timeout, Duration::from_secs(30));
        assert!(c.cors_origins.is_empty() && c.llm.is_none());
    }

    #[test]
    fn every_variable_is_read() {
        let c = from(&[
            ("NALUFX_API_KEY_SHA256", &format!("{HASH}, {HASH}")),
            ("NALUFX_ADDR", "0.0.0.0:9000"),
            ("NALUFX_CORS_ORIGINS", "https://a.example, https://b.example"),
            ("NALUFX_RATE_PER_MINUTE", "100"),
            ("NALUFX_REPORT_RATE_PER_MINUTE", "5"),
            ("NALUFX_TIMEOUT_SECS", "10"),
            ("NALUFX_CONCURRENCY", "8"),
            ("NALUFX_LLM_PROVIDER", "anthropic"),
            ("NALUFX_LLM_MODEL", "claude-sonnet-5-5"),
            ("NALUFX_LLM_MAX_TOKENS", "512"),
        ])
        .unwrap();
        assert_eq!(c.key_hashes.len(), 2);
        assert_eq!(c.addr.port(), 9000);
        assert_eq!(c.cors_origins.len(), 2);
        assert_eq!((c.rate_per_minute, c.report_rate_per_minute, c.concurrency), (100, 5, 8));
        assert_eq!(c.llm.unwrap().max_tokens, 512);
    }

    #[test]
    fn bad_values_are_rejected() {
        for (name, value) in [
            ("NALUFX_API_KEY_SHA256", "nothex"),
            ("NALUFX_API_KEY_SHA256", "abcd"),
            ("NALUFX_ADDR", "nowhere"),
            ("NALUFX_RATE_PER_MINUTE", "0"),
            ("NALUFX_CONCURRENCY", "-1"),
            ("NALUFX_LLM_PROVIDER", "anthropic"),
        ] {
            let mut vars = vec![("NALUFX_API_KEY_SHA256", HASH)];
            vars.retain(|(k, _)| *k != name);
            vars.push((name, value));
            assert!(from(&vars).is_err(), "{name}={value}");
        }
    }
}
