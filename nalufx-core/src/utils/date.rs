use chrono::{DateTime, NaiveDate, TimeZone, Utc};

/// Validates if the input string is a valid date in the format YYYY-MM-DD.
///
/// This function attempts to parse the input string into a `NaiveDate` and then converts it into a `DateTime<Utc>`.
/// If the parsing or conversion fails, it returns an error message.
///
/// # Arguments
///
/// * `input` - A string slice that holds the date to validate.
///
/// # Returns
///
/// This function returns a `Result`:
/// * `Ok(DateTime<Utc>)` - If the input string is a valid date.
/// * `Err(&'static str)` - If the input string is not a valid date.
///
/// # Errors
///
/// The function will return an error if:
/// * The input string cannot be parsed into a date.
/// * The date cannot be converted to a `DateTime<Utc>`.
///
/// # Examples
///
/// ```
/// use nalufx::utils::date::validate_date;
/// use chrono::Utc;
///
/// let valid_date = "2024-01-01";
/// assert!(validate_date(valid_date).is_ok());
///
/// let invalid_date = "2024-13-01";
/// assert!(validate_date(invalid_date).is_err());
/// ```
pub fn validate_date(input: &str) -> Result<DateTime<Utc>, &str> {
    match NaiveDate::parse_from_str(input, "%Y-%m-%d") {
        Ok(date) => match date.and_hms_opt(0, 0, 0) {
            Some(naive_datetime) => match Utc.from_local_datetime(&naive_datetime) {
                chrono::LocalResult::Single(datetime) => Ok(datetime),
                _ => Err("Please enter a valid date in the format YYYY-MM-DD."),
            },
            None => Err("Invalid time component in the date."),
        },
        Err(_) => Err("Please enter a valid date in the format YYYY-MM-DD."),
    }
}
