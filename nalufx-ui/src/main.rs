//! Entry point for the application.
//!
//! This module provides the main function that serves as the entry point
//! for the application, delegating to the `lib_main` function from the
//! `nalufx_ui_lib` crate.

use nalufx_ui::main as lib_main;

/// The entry point of the application.
///
/// This function is responsible for starting the application by calling
/// the `lib_main` function from the `nalufx_ui_lib` crate.
///
/// # Note
///
/// This function delegates to `lib_main` and doesn't return, ensuring
/// the application continues running.
pub fn main() {
    lib_main();
}
