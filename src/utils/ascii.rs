//! This module provides functionality for generating ASCII art from text using the FIGlet library.

use crate::models::error_ascii_art::AsciiArtError;
use figlet_rs::FIGfont;

/// Generates ASCII art from the given text using the standard `FIGfont`.
///
/// # Arguments
///
/// * `text` - The text to convert to ASCII art.
///
/// # Errors
///
/// This function returns an `Err` in the following situations:
///
/// - If the input `text` is empty (`ConversionError`).
/// - If the standard `FIGfont` fails to load (`FontLoadError`).
/// - If the text cannot be converted to ASCII art (`ConversionError`).
///
/// # Examples
///
/// ```
/// use nalufx::utils::ascii::generate_ascii_art;
///
/// let text = "Hello, world!";
/// let result = generate_ascii_art(text);
/// assert!(result.is_ok());
/// ```
pub fn generate_ascii_art(text: &str) -> Result<String, AsciiArtError> {
    if text.is_empty() {
        return Err(AsciiArtError::ConversionError);
    }

    let standard_font = load_standard_font()?;
    let figure = standard_font
        .convert(text)
        .ok_or(AsciiArtError::ConversionError)?;

    Ok(figure.to_string())
}

/// Loads the standard FIGfont.
///
/// # Errors
///
/// This function returns an `Err` if the standard `FIGfont` fails to load (`FontLoadError`).
///
/// # Examples
///
/// ```
/// use nalufx::utils::ascii::load_standard_font;
///
/// let result = load_standard_font();
/// assert!(result.is_ok());
/// ```
pub fn load_standard_font() -> Result<FIGfont, AsciiArtError> {
    FIGfont::standard().map_err(|_| AsciiArtError::FontLoadError)
}
