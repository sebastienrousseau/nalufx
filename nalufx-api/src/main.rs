//! The NaluFX API server. See [`nalufx_api::config`] for its settings.

use nalufx_api::app::{app, AppState};
use nalufx_api::config::ApiConfig;
use nalufx_api::server::serve_until_signal;
use nalufx_llms::http::Http;
use nalufx_llms::provider::{ClientConfig, LlmConfig, LlmProvider};
use nalufx_llms::registry::from_env;
use std::process::ExitCode;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!("{e}");
            ExitCode::FAILURE
        },
    }
}

/// Loads the configuration, builds the provider if one is set, and serves
/// until SIGTERM or Ctrl-C.
async fn run() -> Result<(), String> {
    let config = ApiConfig::from_env()?;
    let provider = provider(&config)?;
    let listener = TcpListener::bind(config.addr)
        .await
        .map_err(|e| format!("cannot listen on {}: {e}", config.addr))?;
    tracing::info!("listening on http://{}", config.addr);
    serve_until_signal(listener, app(AppState::new(config, provider)))
        .await
        .map_err(|e| e.to_string())
}

/// The language model for `/v1/report`, if one is configured.
fn provider(config: &ApiConfig) -> Result<Option<Arc<dyn LlmProvider>>, String> {
    let Some(llm) = &config.llm else {
        tracing::info!("no language model configured: /v1/report answers 503");
        return Ok(None);
    };
    let settings = LlmConfig::new(llm.model.clone(), llm.max_tokens).map_err(|e| e.to_string())?;
    let http = Http::new(ClientConfig::default()).map_err(|e| e.to_string())?;
    let provider = from_env(&llm.provider, settings, http).map_err(|e| e.to_string())?;
    Ok(Some(Arc::from(provider)))
}
