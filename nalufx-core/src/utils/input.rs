use crate::errors::NaluFxError;
use std::io::BufRead;
use std::io::{stdin, stdout, Write};

/// Reads a line of user input from the standard input.
///
/// This function locks the standard input, reads a line of input, and returns it as a `String`.
/// Any I/O errors encountered during the read are mapped to `NaluFxError::InputError`.
///
/// # Returns
///
/// A `Result` containing the user input as a `String` if successful,
/// or an `NaluFxError` if an error occurs during reading.
///
/// # Errors
///
/// Returns an `NaluFxError::InputError` if an error occurs while reading the input.
///
/// # Examples
///
/// ```
/// use nalufx::utils::input::read_user_input;
///
/// match read_user_input() {
///     Ok(input) => println!("User input: {}", input),
///   Err(e) => eprintln!("Error reading input: {}", e),
/// }
/// ```
pub fn read_user_input() -> Result<String, NaluFxError> {
    read_line_from(stdin().lock())
}

/// Reads one line from any reader, line ending included. [`read_user_input`]
/// and [`get_input`] read standard input through it.
///
/// # Errors
///
/// [`NaluFxError::InputError`] if reading fails.
pub fn read_line_from<R: BufRead>(mut reader: R) -> Result<String, NaluFxError> {
    let mut input = String::new();
    let _ = reader.read_line(&mut input).map_err(NaluFxError::InputError)?;
    Ok(input)
}

/// Prompts the user with a message and reads a line of input from the standard input.
///
/// # Arguments
///
/// * `prompt` - A message displayed to the user before reading the input.
///
/// # Returns
///
/// A `Result` containing the user input as a `String` if successful,
/// or an `NaluFxError` if an error occurs during reading.
///
/// # Errors
///
/// Returns an `NaluFxError::InputError` if an error occurs while reading the input.
pub fn get_input(prompt: &str) -> Result<String, NaluFxError> {
    print!("{}", prompt);
    stdout().flush().map_err(NaluFxError::InputError)?;

    Ok(read_line_from(stdin().lock())?.trim().to_string())
}
