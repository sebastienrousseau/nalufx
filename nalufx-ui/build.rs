//! This crate provides build scripts for compiling Slint UI files.
//!
//! It uses the `slint_build` crate to compile Slint files into binary format
//! that can be used in the main application.

/// This function compiles a Slint file into a binary.
///
/// # Arguments
///
/// * `file_path` - A string representing the path to the Slint file to be compiled.
///
/// # Returns
///
/// * `Result<(), Box<dyn std::error::Error>>` - If the compilation is successful, it returns `Ok(())`.
///   If an error occurs during the compilation, it returns `Err` containing the error details.
fn main() {
    slint_build::compile("ui/window.slint").unwrap();
}
