//! `nalufx serve`, `nalufx config check`, and the configuration file's
//! settings for the other commands.

use crate::cli::ServeArgs;
use crate::error::CliError;
use nalufx_api::config::ApiConfig;
use nalufx_api::config_file::{ConfigFile, Source, SETTINGS};
use nalufx_llms::http::Http;
use nalufx_llms::provider::{ClientConfig, LlmConfig};
use nalufx_llms::registry::from_env;
use serde::Serialize;
use std::path::Path;

/// The process environment, as the configuration reads it.
fn env(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// The configuration file at `path`, or none.
///
/// # Errors
///
/// [`CliError::Input`] if the file cannot be read or is not valid.
pub fn file(path: Option<&Path>) -> Result<ConfigFile, CliError> {
    path.map_or_else(|| Ok(ConfigFile::none()), |p| ConfigFile::read(p).map_err(invalid))
}

/// The server's configuration: the file, the environment, then flags.
fn load(path: Option<&Path>) -> Result<(ApiConfig, ConfigFile), CliError> {
    ApiConfig::load_with(path, &env).map_err(invalid)
}

/// A configuration error, with the command that explains it.
fn invalid(message: String) -> CliError {
    CliError::Input {
        message,
        hint: Some(
            "fix that setting; `nalufx config check` shows every setting and where it came from"
                .into(),
        ),
    }
}

/// `nalufx serve`: runs the API server until SIGTERM or Ctrl-C.
///
/// # Errors
///
/// [`CliError::Input`] for a bad setting; [`CliError::Runtime`] if the
/// server cannot start, such as an address already in use.
pub async fn serve(path: Option<&Path>, args: &ServeArgs) -> Result<(), CliError> {
    let (mut config, _) = load(path)?;
    if let Some(addr) = args.addr {
        config.addr = addr;
    }
    nalufx_api::run::run(config).await.map_err(|message| CliError::Runtime {
        message,
        hint: Some("docs/runbook.md says what each setting does".into()),
    })
}

/// One setting as `nalufx config check` reports it.
#[derive(Debug, Serialize)]
struct Setting {
    /// The key in `nalufx.toml`.
    key: &'static str,
    /// The environment variable.
    variable: &'static str,
    /// The value, with key hashes shown as ids.
    value: String,
    /// Where it came from: default, file or environment.
    source: String,
}

/// What `nalufx config check` reports.
#[derive(Debug, Serialize)]
struct Check {
    /// The configuration file, if one was read.
    file: Option<String>,
    /// Every setting.
    settings: Vec<Setting>,
    /// The configured provider's key variable and whether it is set.
    provider_key: Option<ProviderKey>,
}

/// A provider's key variable, and whether it holds a value.
#[derive(Debug, Serialize)]
struct ProviderKey {
    /// The variable, such as `ANTHROPIC_API_KEY`.
    variable: &'static str,
    /// Whether it is set; its value is never shown.
    set: bool,
}

/// `nalufx config check`: validates the server's configuration and shows
/// it.
///
/// # Errors
///
/// [`CliError::Input`] naming the first bad setting.
pub fn check(path: Option<&Path>, json: bool) -> Result<String, CliError> {
    let (config, file) = load(path)?;
    provider_starts(&config)?;
    let check = Check {
        file: file.path().map(|p| p.display().to_string()),
        settings: settings(&config, &file),
        provider_key: provider_key(&config),
    };
    if json {
        return serde_json::to_string_pretty(&check).map_err(|e| {
            CliError::input(format!("cannot write JSON: {e}"), "report this as a bug")
        });
    }
    Ok(check_text(&check))
}

/// Every setting, its value and where it came from.
fn settings(config: &ApiConfig, file: &ConfigFile) -> Vec<Setting> {
    let values = config.describe();
    SETTINGS
        .iter()
        .map(|&(variable, key)| Setting {
            key,
            variable,
            value: values
                .iter()
                .find(|(v, _)| *v == variable)
                .map_or_else(String::new, |(_, x)| x.clone()),
            source: source_name(file.source(variable, &env)),
        })
        .collect()
}

/// The configured provider's key variable, and whether it is set.
fn provider_key(config: &ApiConfig) -> Option<ProviderKey> {
    let variable = crate::commands::key_variable(&config.llm.as_ref()?.provider)?;
    Some(ProviderKey { variable, set: env(variable).is_some_and(|k| !k.trim().is_empty()) })
}

/// Builds the language model provider as the server would, without a
/// request, so a missing key or an unknown provider fails the check
/// rather than the server's start.
fn provider_starts(config: &ApiConfig) -> Result<(), CliError> {
    let Some(llm) = &config.llm else { return Ok(()) };
    let settings = LlmConfig::new(llm.model.clone(), llm.max_tokens)?;
    let http = Http::new(ClientConfig::default())?;
    from_env(&llm.provider, settings, http).map(drop).map_err(CliError::from)
}

/// A source's name in the report.
fn source_name(source: Source) -> String {
    source.to_string()
}

/// The check as a table for people.
fn check_text(check: &Check) -> String {
    let mut t = crate::commands::table(&["Setting", "Variable", "Value", "From"]);
    for s in &check.settings {
        let _ = t.add_row(vec![s.key, s.variable, &s.value, &s.source]);
    }
    let file = check.file.as_deref().map_or_else(
        || "No configuration file (pass --config or set NALUFX_CONFIG).".to_string(),
        |f| format!("Configuration file: {f}"),
    );
    let key = check.provider_key.as_ref().map_or(String::new(), |k| {
        let state = if k.set { "is set" } else { "is not set" };
        format!("\nThe provider's key, {}, {state}.", k.variable)
    });
    format!("{file}\n{t}{key}\nThe configuration is valid.")
}
