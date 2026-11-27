use tracing::error;

/// Validates if the input string can be parsed into a positive float.
///
/// This function checks if the input string can be parsed into a float and if the parsed value is positive.
/// If the input meets these criteria, it returns the parsed float wrapped in `Ok`.
/// Otherwise, it returns an error message.
///
/// # Arguments
///
/// * `input` - A string slice that holds the value to validate.
///
/// # Returns
///
/// This function returns a `Result`:
/// * `Ok(f64)` - If the input string is a valid positive float.
/// * `Err(&'static str)` - If the input string is not a valid positive float.
///
/// # Errors
///
/// The function will return an error if:
/// * The input string cannot be parsed into a float.
/// * The parsed float is not positive.
///
/// # Examples
///
/// ```
/// use nalufx::utils::validation::validate_positive_float;
///
/// let valid_float = "123.45";
/// assert!(validate_positive_float(valid_float).is_ok());
///
/// let invalid_float_non_numeric = "abc";
/// assert!(validate_positive_float(invalid_float_non_numeric).is_err());
///
/// let invalid_float_negative = "-123.45";
/// assert!(validate_positive_float(invalid_float_negative).is_err());
/// ```
pub fn validate_positive_float(input: &str) -> Result<f64, &str> {
    match input.parse::<f64>() {
        Ok(value) if value > 0.0 => Ok(value),
        Ok(_) => {
            error!("Validation failed: The number is not positive. Found: {}", input);
            Err("The number is not positive.")
        },
        Err(_) => {
            error!("Validation failed: The input is not a valid float. Found: {}", input);
            Err("The input is not a valid float.")
        },
    }
}

/// Provides a detailed error message for float validation failures.
///
/// # Arguments
///
/// * `input` - A string slice that holds the value that failed validation.
///
/// # Returns
///
/// A string describing the reason for validation failure.
///
/// # Examples
///
/// ```
/// use nalufx::utils::validation::get_float_validation_error_message;
///
/// let invalid_float_non_numeric = "abc";
/// let error_message = get_float_validation_error_message(invalid_float_non_numeric);
/// assert_eq!(error_message, "The input is not a valid float.");
///
/// let invalid_float_negative = "-123.45";
/// let error_message = get_float_validation_error_message(invalid_float_negative);
/// assert_eq!(error_message, "The number is not positive.");
/// ```
pub fn get_float_validation_error_message(input: &str) -> &'static str {
    match input.parse::<f64>() {
        Ok(value) if value > 0.0 => "No validation error.",
        Ok(_) => "The number is not positive.",
        Err(_) => "The input is not a valid float.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_positive_number_is_accepted() {
        assert_eq!(validate_positive_float("2.5"), Ok(2.5));
        assert_eq!(get_float_validation_error_message("2.5"), "No validation error.");
    }

    #[test]
    fn a_number_that_is_not_positive_says_so() {
        for input in ["0", "-3"] {
            assert_eq!(
                validate_positive_float(input),
                Err("The number is not positive."),
                "{input}"
            );
            assert_eq!(get_float_validation_error_message(input), "The number is not positive.");
        }
    }

    #[test]
    fn text_that_is_not_a_number_says_so() {
        assert_eq!(validate_positive_float("abc"), Err("The input is not a valid float."));
        assert_eq!(get_float_validation_error_message("abc"), "The input is not a valid float.");
    }
}
