//! The NaluFX API server. See [`nalufx_api::config`] for its settings.

use nalufx_api::app::{app, AppState};
use nalufx_api::audit::FileSink;
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
    let config = match ApiConfig::from_env() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("nalufx-api: {e}");
            return ExitCode::FAILURE;
        },
    };
    let telemetry = match nalufx_api::telemetry::init(config.log_format, config.log_sink) {
        Ok(telemetry) => telemetry,
        Err(e) => {
            eprintln!("nalufx-api: {e}");
            return ExitCode::FAILURE;
        },
    };
    let code = match run(config).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!("{e}");
            ExitCode::FAILURE
        },
    };
    telemetry.shutdown();
    code
}

/// Loads the configuration, builds the provider if one is set, and serves
/// until SIGTERM or Ctrl-C.
async fn run(config: ApiConfig) -> Result<(), String> {
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
