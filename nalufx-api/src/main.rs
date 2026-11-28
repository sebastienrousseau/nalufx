//! The NaluFX API server. See [`nalufx_api::config`] and
//! [`nalufx_api::config_file`] for its settings: `NALUFX_CONFIG` names a
//! `nalufx.toml`, and environment variables override it.

use nalufx_api::config::ApiConfig;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let config = match ApiConfig::load(None) {
        Ok((config, _)) => config,
        Err(e) => {
            eprintln!("nalufx-api: {e}");
            return ExitCode::FAILURE;
        },
    };
    match nalufx_api::run::run(config).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("nalufx-api: {e}");
            ExitCode::FAILURE
        },
    }
}
