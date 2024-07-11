//! # `NaluFx` 🦀
//!
//! NaluFX (NFX) is a Rust library that provides sophisticated tools for financial modelling, data fetching, and processing, with a focus on optimizing cash allocation across complex fund structures. It leverages AI-driven predictions and suggestions to enhance decision-making processes.
//!
//! [![NaluFx Logo](https://kura.pro/nalufx/images/banners/banner-nalufx.svg)](https://nalufx.com "A Robust Rust Library for optimizing cash allocation across complex fund structures using AI-driven predictions and suggestions")
//!
//! ## Features
//!
//! - API handlers for processing cash flow requests and generating predictions
//! - Configuration settings for customizing the behaviour of the library
//! - Error handling and custom error types
//! - Data models for representing cash flow requests and responses
//! - Service layer with business logic for fetching data and performing calculations
//! - Utility functions and helpers for common tasks
//!
//! ## Usage
//!
//! To use the Nalufx library in your project, add the following to your `Cargo.toml` file:
//!
//! ```toml
//! [dependencies]
//! nalufx = "0.0.1"
//! ```
//!
//! Then, you can use the modules and functions provided by the library in your code:
//!
//! ```rust
//! use nalufx::api::handlers::predict_cash_flow;
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

/// This module contains the API handlers and related functionality.
pub mod api;

/// This module contains the configuration settings and related functionality.
pub mod config;

/// This module contains error definitions and handling functionality.
pub mod errors;

/// This module contains the generators for ASCII art.
pub mod macros;

/// This module contains data models used throughout the application.
pub mod models;

/// This module contains the service layer with business logic.
pub mod services;

/// This module contains utility functions and helpers.
pub mod utils;
