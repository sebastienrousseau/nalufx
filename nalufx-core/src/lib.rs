//! # `NaluFx` 🦀
//!
//! NaluFX (NFX) forecasts fund cash flows and allocates cash across a forecast horizon. Every number it returns is deterministic, and anything it cannot compute is an explicit error. Its forecasts and allocations are for research and analysis, not financial advice.
//!
//! [![NaluFx Logo](https://kura.pro/nalufx/images/banners/banner-nalufx.svg)](https://nalufx.com "NaluFX: deterministic cash flow forecasting and allocation")
//!
//! ## Features
//!
//! - Cash flow forecasts with a 95% prediction interval (AutoETS)
//! - Inverse-volatility allocation across the forecast horizon
//! - The HTTP API is the `nalufx-api` crate
//! - Error handling and custom error types
//! - Data models for representing cash flow requests and responses
//! - Service layer with business logic for fetching data and performing calculations
//! - Utility functions and helpers for common tasks
//!
//! ## Usage
//!
//! NaluFX is not on crates.io yet; depend on it from Git:
//!
//! ```toml
//! [dependencies]
//! nalufx = { git = "https://github.com/sebastienrousseau/nalufx", tag = "v0.0.5", package = "nalufx-core" }
//! ```
//!
//! Then, you can use the modules and functions provided by the library in your code:
//!
//! ```rust
//! use nalufx::services::fetch_data_svc::fetch_data;
//! use nalufx::utils::calculations::calculate_optimal_allocation;
//! ```
//!
//! For more detailed usage examples and documentation, please refer to the individual module documentation.
#![doc(
    html_favicon_url = "https://kura.pro/nalufx/images/favicon.ico",
    html_logo_url = "https://kura.pro/nalufx/images/logos/nalufx.svg",
    html_root_url = "https://docs.rs/nalufx"
)]
#![crate_name = "nalufx"]
#![crate_type = "lib"]

/// This module contains error definitions and handling functionality.
pub mod errors;

/// This module contains the generators for ASCII art.
pub mod macros;

// The validation and feature macros moved to nalufx-engine in 0.0.6;
// re-exported so `nalufx::check_empty_inputs!` and the rest still work.
pub use nalufx_engine::{
    check_empty_inputs, check_input_lengths, check_invalid_data, check_outliers, extract_features,
    fill_feature_matrix, handle_result, normalize_features,
};

/// This module contains data models used throughout the application.
pub mod models;

/// This module contains the service layer with business logic.
pub mod services;

/// This module contains utility functions and helpers.
pub mod utils;

/// The README's Rust examples, compiled and run as doc tests so they
/// cannot drift from the code.
#[cfg(doctest)]
#[doc = include_str!("../../README.md")]
struct ReadmeExamples;
