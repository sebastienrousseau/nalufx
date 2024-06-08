//! # NaluFx Example
//!
//! This example demonstrates how to use the `services` module from the NaluFx example library.
//! The `services` module includes functions for processing and calculating financial data.
//!
//! ## Usage
//!
//! To run this example, use the following command:
//! ```bash
//! cargo run --example nalufx
//! ```
//!
//! This example will execute the `main` function from the `services` module.
//!
//! ## Modules
//!
//! - `services`: Contains the main function and other utilities for financial data processing.

/// Examples for the `services` module.
pub mod services;

pub(crate) fn main() {
    let _ = services::main();
}
