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
//! | `NALUFX_LOG_FORMAT` | `json` | `json` lines or `text` (logfmt); the level comes from `RUST_LOG`. |
//! | `NALUFX_LOG_SINK` | `stdout` | `stdout`, or `native` for journald (Linux) or os_log (macOS). |
//! | `NALUFX_AUDIT_LOG` | unset | A file to append one JSON line to per allocation and report; unset, no audit log. |
//!
//! Keys are stored only as hashes. Hash one with
//! `printf %s "$KEY" | sha256sum`.

use crate::config_file::{ConfigFile, CONFIG_VAR};
use crate::telemetry::{key_id, LogFormat, LogSink};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
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
    /// How log lines are written.
    pub log_format: LogFormat,
    /// Where log lines go.
    pub log_sink: LogSink,
    /// The audit log's file, if there is one.
    pub audit_log: Option<PathBuf>,
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
            log_format: LogFormat::Json,
            log_sink: LogSink::Stdout,
            audit_log: None,
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

    /// Reads the configuration file, then the environment, which takes
    /// precedence. The file is `path`, or the one `NALUFX_CONFIG` names, or
    /// none. The file comes back too, to say where each setting came from.
    ///
    /// # Errors
    ///
    /// As [`ApiConfig::from_env`], naming the file's key for a value the
    /// file set; or the file cannot be read or holds an unknown key.
    pub fn load(path: Option<&Path>) -> Result<(Self, ConfigFile), String> {
        Self::load_with(path, &|name: &str| std::env::var(name).ok())
    }

    /// As [`ApiConfig::load`], with `env` in place of the environment.
    ///
    /// # Errors
    ///
    /// As [`ApiConfig::load`].
    pub fn load_with(
        path: Option<&Path>,
        env: &impl Fn(&str) -> Option<String>,
    ) -> Result<(Self, ConfigFile), String> {
        let named = env(CONFIG_VAR).filter(|p| !p.trim().is_empty()).map(PathBuf::from);
        let file = match path.map(Path::to_path_buf).or(named) {
            Some(path) => ConfigFile::read(&path)?,
            None => ConfigFile::none(),
        };
        let config = Self::from_lookup(file.lookup(env)).map_err(|e| file.explain(&e, env))?;
        Ok((config, file))
    }

    /// Every setting's value as `nalufx config check` shows it, by
    /// environment variable. Key hashes are shown as key ids.
    #[must_use]
    pub fn describe(&self) -> Vec<(&'static str, String)> {
        let or_none = |v: Option<String>| v.unwrap_or_else(|| "none".to_string());
        let llm = self.llm.as_ref();
        let ids: Vec<String> = self.key_hashes.iter().map(key_id).collect();
        vec![
            ("NALUFX_ADDR", self.addr.to_string()),
            ("NALUFX_API_KEY_SHA256", format!("{} key(s), ids {}", ids.len(), ids.join(", "))),
            (
                "NALUFX_CORS_ORIGINS",
                or_none(Some(self.cors_origins.join(", ")).filter(|o| !o.is_empty())),
            ),
            ("NALUFX_RATE_PER_MINUTE", self.rate_per_minute.to_string()),
            ("NALUFX_REPORT_RATE_PER_MINUTE", self.report_rate_per_minute.to_string()),
            ("NALUFX_TIMEOUT_SECS", self.timeout.as_secs().to_string()),
            ("NALUFX_CONCURRENCY", self.concurrency.to_string()),
            ("NALUFX_LLM_PROVIDER", or_none(llm.map(|l| l.provider.clone()))),
            ("NALUFX_LLM_MODEL", or_none(llm.map(|l| l.model.clone()))),
            ("NALUFX_LLM_MAX_TOKENS", or_none(llm.map(|l| l.max_tokens.to_string()))),
            ("NALUFX_LOG_FORMAT", self.log_format.to_string()),
            ("NALUFX_LOG_SINK", self.log_sink.to_string()),
            ("NALUFX_AUDIT_LOG", or_none(self.audit_log.as_ref().map(|p| p.display().to_string()))),
        ]
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
        config.read_operations(&get)?;
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

    /// The log format and the audit log.
    fn read_operations(&mut self, get: &impl Fn(&str) -> Option<String>) -> Result<(), String> {
        if let Some(format) = get("NALUFX_LOG_FORMAT") {
            self.log_format = format.parse()?;
        }
        if let Some(sink) = get("NALUFX_LOG_SINK") {
            self.log_sink = sink.parse()?;
        }
        self.audit_log = get("NALUFX_AUDIT_LOG").map(PathBuf::from);
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
        assert!(c.cors_origins.is_empty() && c.llm.is_none() && c.audit_log.is_none());
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
            ("NALUFX_LOG_FORMAT", "text"),
            ("NALUFX_LOG_SINK", "native"),
            ("NALUFX_AUDIT_LOG", "/var/log/nalufx/audit.jsonl"),
        ])
        .unwrap();
        assert_eq!(c.key_hashes.len(), 2);
        assert_eq!(c.addr.port(), 9000);
        assert_eq!(c.cors_origins.len(), 2);
        assert_eq!((c.rate_per_minute, c.report_rate_per_minute, c.concurrency), (100, 5, 8));
        assert_eq!(c.llm.unwrap().max_tokens, 512);
        assert_eq!(c.log_format, LogFormat::Text);
        assert_eq!(c.log_sink, LogSink::Native);
        assert_eq!(c.audit_log, Some(PathBuf::from("/var/log/nalufx/audit.jsonl")));
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
            ("NALUFX_LOG_FORMAT", "xml"),
        ] {
            let mut vars = vec![("NALUFX_API_KEY_SHA256", HASH)];
            vars.retain(|(k, _)| *k != name);
            vars.push((name, value));
            assert!(from(&vars).is_err(), "{name}={value}");
        }
    }

    /// A `nalufx.toml` in a fresh directory, and its path.
    fn toml(text: &str) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nalufx.toml");
        std::fs::write(&path, text).unwrap();
        (dir, path)
    }

    #[test]
    fn the_file_sets_and_the_environment_overrides() {
        let (_dir, path) = toml(&format!(
            "[server]\naddr = \"0.0.0.0:9000\"\napi_key_sha256 = [\"{HASH}\"]\n\
             [limits]\nconcurrency = 8\n[log]\nformat = \"text\"\n"
        ));
        let env = |v: &str| (v == "NALUFX_CONCURRENCY").then(|| "16".to_string());
        let (c, file) = ApiConfig::load_with(Some(&path), &env).unwrap();
        assert_eq!(c.addr.to_string(), "0.0.0.0:9000");
        assert_eq!(c.concurrency, 16);
        assert_eq!(c.log_format, LogFormat::Text);
        assert_eq!(c.rate_per_minute, 600);
        assert_eq!(file.path(), Some(path.as_path()));
    }

    #[test]
    fn nalufx_config_names_the_file_when_no_path_is_given() {
        let (_dir, path) = toml(&format!("[server]\napi_key_sha256 = [\"{HASH}\"]\n"));
        let named = path.display().to_string();
        let env = move |v: &str| (v == "NALUFX_CONFIG").then(|| named.clone());
        assert!(ApiConfig::load_with(None, &env).is_ok());
        let none = |_: &str| None;
        assert!(ApiConfig::load_with(None, &none).unwrap_err().contains("NALUFX_API_KEY_SHA256"));
    }

    #[test]
    fn a_bad_value_in_the_file_is_named_by_its_key() {
        let (_dir, path) =
            toml(&format!("[server]\napi_key_sha256 = [\"{HASH}\"]\n[limits]\ntimeout_secs = 0\n"));
        let err = ApiConfig::load_with(Some(&path), &|_: &str| None).unwrap_err();
        assert!(err.contains("limits.timeout_secs in "), "{err}");
        assert!(!err.contains("NALUFX_TIMEOUT_SECS"), "{err}");
        let missing = ApiConfig::load_with(Some(Path::new("/no/such.toml")), &|_: &str| None);
        assert!(missing.unwrap_err().contains("cannot read /no/such.toml"));
    }

    #[test]
    fn describe_shows_key_ids_never_hashes() {
        let c = from(&[("NALUFX_API_KEY_SHA256", HASH)]).unwrap();
        let shown: Vec<String> = c.describe().into_iter().map(|(_, v)| v).collect();
        assert!(shown.iter().any(|v| v == "1 key(s), ids 9f86d081"), "{shown:?}");
        assert!(!shown.iter().any(|v| v.contains(HASH)), "{shown:?}");
        assert_eq!(c.describe().len(), crate::config_file::SETTINGS.len());
    }
}
