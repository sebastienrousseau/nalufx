use log::error;

/// Validates if the input string is a non-empty alphanumeric ticker symbol.
///
/// This function checks if the input string is non-empty and consists only of alphanumeric characters.
/// If the input string meets these criteria, it returns the input string wrapped in `Ok`.
/// Otherwise, it returns an error message.
///
/// # Arguments
///
/// * `input` - A string slice that holds the ticker symbol to validate.
///
/// # Returns
///
/// This function returns a `Result`:
/// * `Ok(&str)` - If the input string is a valid ticker symbol.
/// * `Err(&'static str)` - If the input string is not a valid ticker symbol.
///
/// # Errors
///
/// The function will return an error if:
/// * The input string is empty.
/// * The input string contains non-alphanumeric characters.
///
/// # Examples
///
/// ```
/// use nalufx::utils::ticker::validate_ticker;
///
/// let valid_ticker = "AAPL";
/// assert!(validate_ticker(valid_ticker).is_ok());
///
/// let invalid_ticker_empty = "";
/// assert!(validate_ticker(invalid_ticker_empty).is_err());
///
/// let invalid_ticker_non_alphanumeric = "AAPL$";
/// assert!(validate_ticker(invalid_ticker_non_alphanumeric).is_err());
/// ```
pub fn validate_ticker(input: &str) -> Result<&str, &str> {
    if input.is_empty() {
        error!("Validation failed: The ticker symbol cannot be empty.");
        Err("The ticker symbol cannot be empty.")
    } else if !input.chars().all(|c| c.is_alphanumeric()) {
        error!("Validation failed: The ticker symbol must be alphanumeric. Found: {}", input);
        Err("The ticker symbol must be alphanumeric.")
    } else {
        Ok(input)
    }
}

/// Provides a detailed error message for ticker validation failures.
///
/// # Arguments
///
/// * `input` - A string slice that holds the ticker symbol that failed validation.
///
/// # Returns
///
/// A string describing the reason for validation failure.
///
/// # Examples
///
/// ```
/// use nalufx::utils::ticker::get_validation_error_message;
///
/// let invalid_ticker_empty = "";
/// let error_message = get_validation_error_message(invalid_ticker_empty);
/// assert_eq!(error_message, "The ticker symbol cannot be empty.");
///
/// let invalid_ticker_non_alphanumeric = "AAPL$";
/// let error_message = get_validation_error_message(invalid_ticker_non_alphanumeric);
/// assert_eq!(error_message, "The ticker symbol must be alphanumeric.");
/// ```
pub fn get_validation_error_message(input: &str) -> &'static str {
    if input.is_empty() {
        "The ticker symbol cannot be empty."
    } else if !input.chars().all(|c| c.is_alphanumeric()) {
        "The ticker symbol must be alphanumeric."
    } else {
        "Unknown validation error."
    }
}
