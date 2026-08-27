//! Integration tests for `nalufx-core`.
//!
//! This tree previously sat at the repository root, where the virtual
//! workspace manifest meant cargo never built it and none of these
//! tests ran.

/// This module contains the tests for the `api` module.
pub mod api;

/// This module contains the tests for the `ascii` module.
pub mod macros;

/// This module contains the tests for the `utils` module.
pub mod utils;
