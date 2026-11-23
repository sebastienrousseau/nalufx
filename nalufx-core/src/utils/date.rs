//! Calendar-date parsing and formatting on top of the `time` crate.
//!
//! These functions keep the rules nalufx had when it used chrono's
//! `NaiveDate::parse_from_str(input, "%Y-%m-%d")` and
//! `format("%Y-%m-%d")`, so moving library changed no accepted input
//! and no rendered date:
//!
//! * leading whitespace before each number is skipped;
//! * a month or day is one or two digits, so `2024-1-1` is accepted;
//! * a year is one to four digits, or any number of digits after an
//!   explicit `+` or `-` sign;
//! * the year must lie in chrono's range, -262143 to 262142;
//! * nothing may follow the day.

use time::{Date, Month, OffsetDateTime};

/// The message every rejected input produces.
const INVALID_DATE: &str = "Please enter a valid date in the format YYYY-MM-DD.";

/// The earliest year a date may have.
const MIN_YEAR: i64 = -262_143;

/// The latest year a date may have.
const MAX_YEAR: i64 = 262_142;

/// Validates if the input string is a valid date in the format YYYY-MM-DD.
///
/// This function parses the input string into a calendar date and returns
/// midnight UTC on that date. If the parsing fails, it returns an error
/// message.
///
/// # Arguments
///
/// * `input` - A string slice that holds the date to validate.
///
/// # Returns
///
/// This function returns a `Result`:
/// * `Ok(OffsetDateTime)` - Midnight UTC on the date, if the input is valid.
/// * `Err(&'static str)` - If the input string is not a valid date.
///
/// # Errors
///
/// The function will return an error if the input string cannot be
/// parsed into a calendar date.
///
/// # Examples
///
/// ```
/// use nalufx::utils::date::validate_date;
///
/// let valid_date = "2024-01-01";
/// assert!(validate_date(valid_date).is_ok());
///
/// let invalid_date = "2024-13-01";
/// assert!(validate_date(invalid_date).is_err());
/// ```
pub fn validate_date(input: &str) -> Result<OffsetDateTime, &str> {
    parse_ymd(input).map(|date| date.midnight().assume_utc()).ok_or(INVALID_DATE)
}

/// Formats the calendar date of `datetime` as `YYYY-MM-DD`.
///
/// Years outside 0 to 9999 carry an explicit sign and at least four
/// digits (`+10000-01-01`, `-0001-01-01`).
///
/// # Examples
///
/// ```
/// use nalufx::utils::date::{format_date, validate_date};
///
/// let date = validate_date("2024-1-5").unwrap();
/// assert_eq!(format_date(date), "2024-01-05");
/// ```
pub fn format_date(datetime: OffsetDateTime) -> String {
    let year = datetime.year();
    let month = u8::from(datetime.month());
    let day = datetime.day();
    if (0..=9999).contains(&year) {
        format!("{year:04}-{month:02}-{day:02}")
    } else {
        format!("{year:+05}-{month:02}-{day:02}")
    }
}

/// Parses `YYYY-MM-DD` with the rules listed in the module documentation.
fn parse_ymd(input: &str) -> Option<Date> {
    let (year, rest) = scan_year(input.trim_start())?;
    let (month, rest) = scan_digits(rest.strip_prefix('-')?.trim_start(), 2)?;
    let (day, rest) = scan_digits(rest.strip_prefix('-')?.trim_start(), 2)?;
    if !rest.is_empty() || !(MIN_YEAR..=MAX_YEAR).contains(&year) {
        return None;
    }
    let month = Month::try_from(u8::try_from(month).ok()?).ok()?;
    Date::from_calendar_date(i32::try_from(year).ok()?, month, u8::try_from(day).ok()?).ok()
}

/// Reads a year: up to four digits, or any number after a sign.
fn scan_year(input: &str) -> Option<(i64, &str)> {
    if let Some(rest) = input.strip_prefix('-') {
        let (value, rest) = scan_digits(rest, usize::MAX)?;
        Some((-value, rest))
    } else if let Some(rest) = input.strip_prefix('+') {
        scan_digits(rest, usize::MAX)
    } else {
        scan_digits(input, 4)
    }
}

/// Reads between one and `max` ASCII digits from the start of `input`.
///
/// Returns `None` when there is no digit or the value overflows `i64`.
fn scan_digits(input: &str, max: usize) -> Option<(i64, &str)> {
    let len = input.bytes().take(max).take_while(u8::is_ascii_digit).count();
    if len == 0 {
        return None;
    }
    let value = input[..len]
        .bytes()
        .try_fold(0i64, |acc, digit| acc.checked_mul(10)?.checked_add(i64::from(digit - b'0')))?;
    Some((value, &input[len..]))
}
