//! The configuration file, `nalufx.toml`, and where each setting came
//! from.
//!
//! Every setting has an environment variable and a key in the file:
//!
//! ```toml
//! [server]
//! addr = "127.0.0.1:8080"
//! api_key_sha256 = ["<hex SHA-256 of a key>"]
//! cors_origins = ["https://app.example"]
//!
//! [limits]
//! rate_per_minute = 600
//! report_rate_per_minute = 30
//! timeout_secs = 30
//! concurrency = 64
//!
//! [llm]
//! provider = "anthropic"
//! model = "claude-sonnet-5-5"
//! max_tokens = 2048
//!
//! [log]
//! format = "json"
//! sink = "stdout"
//!
//! [audit]
//! log = "/var/log/nalufx/audit.jsonl"
//! ```
//!
//! The defaults come first, then the file, then the environment, so a
//! variable overrides the file. Provider keys (`ANTHROPIC_API_KEY`, ...)
//! are read from the environment only and have no place in the file.

use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};

/// The variable naming the configuration file, when no path is given.
pub const CONFIG_VAR: &str = "NALUFX_CONFIG";

/// Every setting: its environment variable and its key in the file.
pub const SETTINGS: [(&str, &str); 13] = [
    ("NALUFX_ADDR", "server.addr"),
    ("NALUFX_API_KEY_SHA256", "server.api_key_sha256"),
    ("NALUFX_CORS_ORIGINS", "server.cors_origins"),
    ("NALUFX_RATE_PER_MINUTE", "limits.rate_per_minute"),
    ("NALUFX_REPORT_RATE_PER_MINUTE", "limits.report_rate_per_minute"),
    ("NALUFX_TIMEOUT_SECS", "limits.timeout_secs"),
    ("NALUFX_CONCURRENCY", "limits.concurrency"),
    ("NALUFX_LLM_PROVIDER", "llm.provider"),
    ("NALUFX_LLM_MODEL", "llm.model"),
    ("NALUFX_LLM_MAX_TOKENS", "llm.max_tokens"),
    ("NALUFX_LOG_FORMAT", "log.format"),
    ("NALUFX_LOG_SINK", "log.sink"),
    ("NALUFX_AUDIT_LOG", "audit.log"),
];

/// Where a setting's value came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// Nothing set it.
    Default,
    /// The configuration file.
    File,
    /// An environment variable.
    Env,
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Default => "default",
            Self::File => "file",
            Self::Env => "environment",
        })
    }
}

/// The settings a configuration file holds, by environment variable name.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConfigFile {
    path: Option<PathBuf>,
    values: HashMap<&'static str, String>,
}

impl ConfigFile {
    /// No file: every setting comes from the environment or its default.
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }

    /// Reads and checks the file at `path`.
    ///
    /// # Errors
    ///
    /// A message naming the file and, for a bad setting, its key: the file
    /// cannot be read, is not TOML, has a key nalufx does not know, or a
    /// value of the wrong type.
    pub fn read(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let mut file = Self::parse(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        file.path = Some(path.to_path_buf());
        Ok(file)
    }

    /// Checks TOML text and keeps its settings.
    ///
    /// # Errors
    ///
    /// As [`ConfigFile::read`], without the file's name.
    pub fn parse(text: &str) -> Result<Self, String> {
        let table: toml::Table = text.parse().map_err(|e: toml::de::Error| e.to_string())?;
        let mut entries = Vec::new();
        for (section, keys) in &table {
            let toml::Value::Table(keys) = keys else {
                return Err(format!("`{section}` must be a section, such as [server]"));
            };
            for (key, value) in keys {
                let name = format!("{section}.{key}");
                let var = var_for(&name).ok_or_else(|| unknown(&name))?;
                entries.push((var, text_of(&name, value)?));
            }
        }
        Ok(Self { path: None, values: entries.into_iter().collect() })
    }

    /// The file's path, if there is a file.
    #[must_use]
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// The file's value for an environment variable's setting.
    #[must_use]
    pub fn get(&self, var: &str) -> Option<&str> {
        self.values.get(var).map(String::as_str)
    }

    /// Where `var`'s value comes from, given the environment `env`.
    #[must_use]
    pub fn source(&self, var: &str, env: &impl Fn(&str) -> Option<String>) -> Source {
        if set(env(var)).is_some() {
            Source::Env
        } else if self.values.contains_key(var) {
            Source::File
        } else {
            Source::Default
        }
    }

    /// A lookup that takes the environment first, then the file.
    pub fn lookup<'a>(
        &'a self,
        env: &'a impl Fn(&str) -> Option<String>,
    ) -> impl Fn(&str) -> Option<String> + 'a {
        move |var| set(env(var)).or_else(|| self.get(var).map(str::to_string))
    }

    /// `message` with each variable the file set replaced by its key in
    /// the file, so an error names what the reader wrote.
    #[must_use]
    pub fn explain(&self, message: &str, env: &impl Fn(&str) -> Option<String>) -> String {
        let Some(path) = &self.path else { return message.to_string() };
        // No variable's name is part of another's, so the order is free.
        SETTINGS
            .iter()
            .filter(|(var, _)| self.source(var, env) == Source::File)
            .fold(message.to_string(), |text, (var, key)| {
                text.replace(var, &format!("{key} in {}", path.display()))
            })
    }
}

/// A value that is set and not blank.
fn set(value: Option<String>) -> Option<String> {
    value.filter(|v| !v.trim().is_empty())
}

/// The environment variable for a key in the file.
fn var_for(key: &str) -> Option<&'static str> {
    SETTINGS.iter().find(|(_, k)| *k == key).map(|(var, _)| *var)
}

/// The error for a key nalufx does not know.
fn unknown(key: &str) -> String {
    let keys: Vec<&str> = SETTINGS.iter().map(|(_, k)| *k).collect();
    format!("unknown setting `{key}`; the settings are {}", keys.join(", "))
}

/// A value as the text its environment variable would hold: a string, a
/// whole number, or a list of strings joined by commas.
fn text_of(key: &str, value: &toml::Value) -> Result<String, String> {
    match value {
        toml::Value::String(s) => Ok(s.clone()),
        toml::Value::Integer(n) => Ok(n.to_string()),
        toml::Value::Array(items) => items
            .iter()
            .map(|item| match item {
                toml::Value::String(s) if !s.contains(',') => Ok(s.clone()),
                _ => Err(format!("`{key}` must be a list of strings without commas")),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|items| items.join(",")),
        _ => Err(format!("`{key}` must be a string, a whole number or a list of strings")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no_env(_: &str) -> Option<String> {
        None
    }

    #[test]
    fn no_variable_name_is_part_of_another() {
        for (a, _) in SETTINGS {
            for (b, _) in SETTINGS {
                assert!(a == b || !b.contains(a), "{a} is part of {b}");
            }
        }
    }

    #[test]
    fn every_key_maps_to_its_variable() {
        let f = ConfigFile::parse(
            "[server]\naddr = \"0.0.0.0:9000\"\ncors_origins = [\"https://a\", \"https://b\"]\n\
             [limits]\nconcurrency = 8\n[audit]\nlog = \"/tmp/a.jsonl\"\n",
        )
        .unwrap();
        assert_eq!(f.get("NALUFX_ADDR"), Some("0.0.0.0:9000"));
        assert_eq!(f.get("NALUFX_CORS_ORIGINS"), Some("https://a,https://b"));
        assert_eq!(f.get("NALUFX_CONCURRENCY"), Some("8"));
        assert_eq!(f.get("NALUFX_AUDIT_LOG"), Some("/tmp/a.jsonl"));
        assert_eq!(f.get("NALUFX_LLM_MODEL"), None);
    }

    #[test]
    fn unknown_keys_and_wrong_types_are_refused() {
        let err = ConfigFile::parse("[server]\nadress = \"x\"\n").unwrap_err();
        assert!(err.contains("unknown setting `server.adress`"), "{err}");
        assert!(err.contains("server.addr"), "{err}");
        // A provider key has no place in the file.
        assert!(ConfigFile::parse("[llm]\napi_key = \"sk-x\"\n").is_err());
        assert!(ConfigFile::parse("addr = \"x\"\n").unwrap_err().contains("section"));
        assert!(ConfigFile::parse("[log]\nformat = true\n").unwrap_err().contains("log.format"));
        assert!(ConfigFile::parse("[server]\ncors_origins = [\"a,b\"]\n").is_err());
        assert!(ConfigFile::parse("[server\n").is_err());
    }

    #[test]
    fn the_environment_overrides_the_file() {
        let f = ConfigFile::parse("[server]\naddr = \"0.0.0.0:9000\"\n").unwrap();
        let env = |v: &str| (v == "NALUFX_ADDR").then(|| "127.0.0.1:1".to_string());
        assert_eq!(f.lookup(&env)("NALUFX_ADDR").as_deref(), Some("127.0.0.1:1"));
        assert_eq!(f.source("NALUFX_ADDR", &env), Source::Env);
        assert_eq!(f.lookup(&no_env)("NALUFX_ADDR").as_deref(), Some("0.0.0.0:9000"));
        assert_eq!(f.source("NALUFX_ADDR", &no_env), Source::File);
        assert_eq!(f.source("NALUFX_CONCURRENCY", &no_env), Source::Default);
        // A blank variable does not hide the file's value.
        let blank = |_: &str| Some("  ".to_string());
        assert_eq!(f.lookup(&blank)("NALUFX_ADDR").as_deref(), Some("0.0.0.0:9000"));
    }

    #[test]
    fn errors_name_the_files_key() {
        let mut f = ConfigFile::parse("[limits]\nrate_per_minute = 0\n").unwrap();
        f.path = Some(PathBuf::from("nalufx.toml"));
        let message = "NALUFX_RATE_PER_MINUTE must be positive";
        assert_eq!(
            f.explain(message, &no_env),
            "limits.rate_per_minute in nalufx.toml must be positive"
        );
        // Set by the environment instead: the variable's name stays.
        let env = |v: &str| (v == "NALUFX_RATE_PER_MINUTE").then(|| "0".to_string());
        assert_eq!(f.explain(message, &env), message);
        // A similar name the file did not set is left alone.
        let report = "NALUFX_REPORT_RATE_PER_MINUTE must be positive";
        assert_eq!(f.explain(report, &no_env), report);
    }
}
