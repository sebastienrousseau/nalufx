//! # NaluFx
//!
//! The NaluFx HTTP service, on hyper. `POST /predict` forecasts cash
//! flows with AutoETS and allocates across the forecast horizon by inverse
//! volatility; errors are RFC 9457 problem details.
//!
//! ## Features
//! - Cash flow forecasts with a 95% prediction interval
//! - Inverse-volatility allocation across the forecast horizon
//!
//! ## Getting Started
//! To run the application, ensure that you have the necessary environment variables set in a `.env` file:
//!
//! ```env
//! SERVER_ADDR=127.0.0.1:8080
//! ```
//!
//! Then, run the application using the following command:
//!
//! ```bash
//! cargo run
//! ```
//!
//! The server will start and bind to the address specified in the `SERVER_ADDR` environment variable.

use dotenvy::dotenv;
use nalufx::api::server::serve;
use nalufx::config::Config;

/// The main entry point of the application.
///
/// This function initializes the environment, loads the configuration,
/// and starts the HTTP server. It sets up the necessary routes
/// and binds the server to the address specified in the configuration.
///
/// # Returns
///
/// A `std::io::Result<()>` indicating the success or failure of the server execution.
///
/// # Errors
///
/// This function will return an error if the configuration cannot be loaded,
/// or if the server fails to bind to the specified address.
///
/// # Examples
///
/// To run the application, use the following command:
/// ```bash
/// cargo run
/// ```
///
/// Ensure that the `.env` file contains the necessary environment variables:
/// ```env
/// SERVER_ADDR=127.0.0.1:8080
/// ```
#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file, ignoring any error
    if dotenv().is_err() {
        // Ignore error, dotenv will return an error if the file is not found, which is okay
    }
    env_logger::init();

    let config = Config::from_env().expect("Failed to load configuration");

    let addr = config.server_addr.parse().map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Invalid SERVER_ADDR `{}`: {err}", config.server_addr),
        )
    })?;

    serve(addr).await
}
