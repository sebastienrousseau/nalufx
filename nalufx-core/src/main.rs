//! # NaluFx
//!
//! NaluFx is a financial forecasting application built using Actix Web. It provides an API for predicting cash flows and optimizing portfolio allocations.
//!
//! ## Features
//! - Predict cash flows based on historical data
//! - Optimize portfolio allocations
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

use actix_web::{App, HttpServer};
use dotenvy::dotenv;
use nalufx::api::handlers::predict_cash_flow;
use nalufx::config::Config;

/// The main entry point of the application.
///
/// This function initializes the environment, loads the configuration,
/// and starts the Actix web server. It sets up the necessary routes
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
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file, ignoring any error
    if dotenv().is_err() {
        // Ignore error, dotenv will return an error if the file is not found, which is okay
    }
    env_logger::init();

    let config = Config::from_env().expect("Failed to load configuration");

    HttpServer::new(|| App::new().service(predict_cash_flow)).bind(config.server_addr)?.run().await
}
