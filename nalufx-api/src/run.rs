//! Running the server from a configuration, as `nalufx-api` and
//! `nalufx serve` both do.

use crate::app::{app, AppState};
use crate::audit::FileSink;
use crate::config::ApiConfig;
use crate::server::serve_until_signal;
use nalufx_llms::http::Http;
use nalufx_llms::provider::{ClientConfig, LlmConfig, LlmProvider};
use nalufx_llms::registry::from_env;
use std::sync::Arc;
use tokio::net::TcpListener;

/// Starts logging and trace export, builds the language model provider if
/// one is configured, opens the audit log if one is set, and serves until
/// SIGTERM or Ctrl-C. The last spans are flushed before it returns.
///
/// # Errors
///
/// Why the server could not start or stopped: logging could not start,
/// the address is taken, the provider's key is missing, or the audit log
/// cannot be opened. Once logging runs, the error is logged too.
pub async fn run(config: ApiConfig) -> Result<(), String> {
    let telemetry = crate::telemetry::init(config.log_format, config.log_sink)?;
    let result = serve(config).await;
    if let Err(e) = &result {
        tracing::error!("{e}");
    }
    telemetry.shutdown();
    result
}

/// Binds, builds the state and serves.
async fn serve(config: ApiConfig) -> Result<(), String> {
    let provider = provider(&config)?;
    let listener = TcpListener::bind(config.addr)
        .await
        .map_err(|e| format!("cannot listen on {}: {e}", config.addr))?;
    tracing::info!("listening on http://{}", config.addr);
    let mut state = AppState::new(config.clone(), provider);
    if let Some(path) = &config.audit_log {
        state = state.with_audit(Arc::new(FileSink::open(path)?));
        tracing::info!(path = %path.display(), "writing the audit log");
    }
    serve_until_signal(listener, app(state)).await.map_err(|e| e.to_string())
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
