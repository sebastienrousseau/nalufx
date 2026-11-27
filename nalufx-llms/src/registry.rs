//! Building a provider by name, with its key from the environment.

use crate::adapters::anthropic::{self, Anthropic};
use crate::adapters::gemini::{self, Gemini};
use crate::adapters::openai_compatible::{
    Endpoint, OpenAiCompatible, AGY, CUSTOM, MISTRAL, OLLAMA, OPENAI,
};
use crate::error::LlmError;
use crate::http::Http;
use crate::provider::{LlmConfig, LlmProvider};

/// The provider names [`from_env`] accepts.
pub const PROVIDERS: [&str; 7] =
    ["anthropic", "gemini", "openai", "mistral", "ollama", "agy", "openai_compatible"];

/// The environment variable that overrides a provider's base URL, and
/// supplies it for `agy` and `openai_compatible`.
pub const BASE_URL_VAR: &str = "NALUFX_LLM_BASE_URL";

/// Builds the named provider with `config`, reading its API key (and, if
/// set, a base URL override) from the environment or a `.env` file.
///
/// # Errors
///
/// - [`LlmError::Config`] for an unknown name, a provider without a base
///   URL when [`BASE_URL_VAR`] is unset, or a bad configuration.
/// - [`LlmError::MissingApiKey`] if the provider needs a key and it is unset.
pub fn from_env(
    name: &str,
    config: LlmConfig,
    http: Http,
) -> Result<Box<dyn LlmProvider>, LlmError> {
    let base_url = env_var(BASE_URL_VAR);
    match name {
        "anthropic" => anthropic_from_env(base_url, config, http),
        "gemini" => gemini_from_env(base_url, config, http),
        other => Ok(Box::new(openai_compatible(other, base_url, config, http)?)),
    }
}

/// Anthropic, with its key from the environment.
fn anthropic_from_env(
    base_url: Option<String>,
    config: LlmConfig,
    http: Http,
) -> Result<Box<dyn LlmProvider>, LlmError> {
    let adapter =
        Anthropic::new(http, api_key(anthropic::PROVIDER, anthropic::API_KEY_VAR)?, config)?;
    Ok(Box::new(match base_url {
        Some(url) => adapter.with_base_url(url),
        None => adapter,
    }))
}

/// Gemini, with its key from the environment.
fn gemini_from_env(
    base_url: Option<String>,
    config: LlmConfig,
    http: Http,
) -> Result<Box<dyn LlmProvider>, LlmError> {
    let adapter = Gemini::new(http, api_key(gemini::PROVIDER, gemini::API_KEY_VAR)?, config)?;
    Ok(Box::new(match base_url {
        Some(url) => adapter.with_base_url(url),
        None => adapter,
    }))
}

/// An OpenAI-compatible provider by name.
fn openai_compatible(
    name: &str,
    base_url: Option<String>,
    config: LlmConfig,
    http: Http,
) -> Result<OpenAiCompatible, LlmError> {
    let endpoint = endpoint(name)?;
    let url = base_url.or_else(|| endpoint.base_url.map(str::to_string)).ok_or_else(|| {
        LlmError::Config(format!("{name} has no public endpoint: set {BASE_URL_VAR}"))
    })?;
    let key = endpoint.api_key_var.map(|var| api_key(endpoint.provider, var)).transpose()?;
    OpenAiCompatible::new(http, endpoint, url, key, config)
}

/// The OpenAI-compatible endpoint with this name.
fn endpoint(name: &str) -> Result<Endpoint, LlmError> {
    [OPENAI, MISTRAL, OLLAMA, AGY, CUSTOM].into_iter().find(|e| e.provider == name).ok_or_else(
        || {
            LlmError::Config(format!(
                "unknown provider `{name}`; expected one of {}",
                PROVIDERS.join(", ")
            ))
        },
    )
}

/// A provider's API key from the environment, or from a `.env` file.
///
/// # Errors
///
/// [`LlmError::MissingApiKey`] naming the variable if it is unset or empty.
pub fn api_key(provider: &'static str, variable: &'static str) -> Result<String, LlmError> {
    env_var(variable).ok_or(LlmError::MissingApiKey { provider, variable })
}

/// A non-empty environment variable, loading `.env` if it is not set.
fn env_var(name: &str) -> Option<String> {
    let read = || std::env::var(name).ok().filter(|v| !v.trim().is_empty());
    read().or_else(|| dotenvy::dotenv().ok().and_then(|_| read()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::ClientConfig;

    fn config() -> LlmConfig {
        LlmConfig::new("a-model", 256).unwrap()
    }

    fn http() -> Http {
        Http::new(ClientConfig::default()).unwrap()
    }

    #[test]
    fn an_unknown_name_is_a_configuration_error() {
        let err = from_env("magic", config(), http()).err().expect("an error");
        assert!(matches!(&err, LlmError::Config(m) if m.contains("magic")), "{err}");
    }

    #[test]
    fn every_name_maps_to_an_endpoint_or_a_native_adapter() {
        for name in PROVIDERS {
            assert!(matches!(name, "anthropic" | "gemini") || endpoint(name).is_ok(), "{name}");
        }
    }

    #[test]
    fn ollama_needs_no_key_and_has_a_local_default() {
        let p = openai_compatible("ollama", None, config(), http()).unwrap();
        assert_eq!((p.name(), p.model()), ("ollama", "a-model"));
    }

    #[test]
    fn a_provider_without_a_public_endpoint_needs_a_base_url() {
        let err = openai_compatible("openai_compatible", None, config(), http()).unwrap_err();
        assert!(matches!(&err, LlmError::Config(m) if m.contains(BASE_URL_VAR)), "{err}");
        assert!(openai_compatible(
            "openai_compatible",
            Some("http://gpu:8000/v1".into()),
            config(),
            http()
        )
        .is_ok());
    }

    #[test]
    fn a_missing_key_names_its_variable() {
        let err = api_key("anthropic", "NALUFX_TEST_KEY_THAT_IS_NOT_SET").unwrap_err();
        assert_eq!(
            err,
            LlmError::MissingApiKey {
                provider: "anthropic",
                variable: "NALUFX_TEST_KEY_THAT_IS_NOT_SET"
            }
        );
    }
}
