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
    // `unwrap()` here reports a Slint compile failure as a build-script
    // panic, which buries the actual diagnostics in a single-line Debug
    // dump of the whole error vector. Printing them and exiting keeps
    // each `file:line: message` on its own line.
    if let Err(error) = slint_build::compile("ui/window.slint") {
        eprintln!("error: failed to compile ui/window.slint:\n{error}");
        std::process::exit(1);
    }
}
