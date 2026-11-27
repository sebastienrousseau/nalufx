/// Formats a floating-point number as currency in US dollars.
///
/// # Arguments
///
/// * `value` - A floating-point number representing the amount to format.
///
/// # Returns
///
/// A `String` representing the formatted currency value.
///
/// # Examples
///
/// ```
/// use nalufx::utils::currency::format_currency;
///
/// let formatted = format_currency(1234.5678);
/// assert_eq!(formatted, "$1,234.57");
///
/// let formatted_negative = format_currency(-1234.5678);
/// assert_eq!(formatted_negative, "-$1,234.57");
/// ```
pub fn format_currency(value: f64) -> String {
    /// Helper function to format the dollar part with commas.
    fn format_dollars(dollars: i64) -> String {
        let dollars_abs = dollars.abs().to_string();
        let mut result = String::new();

        for (count, digit) in dollars_abs.chars().rev().enumerate() {
            if count > 0 && count % 3 == 0 {
                result.push(',');
            }
            result.push(digit);
        }

        result.chars().rev().collect::<String>()
    }

    let int_value = (value * 100.0).round() as i64; // Convert to integer cents
    let dollars = int_value / 100;
    let cents = (int_value % 100).abs(); // Absolute value for cents
    let formatted_dollars = format_dollars(dollars);

    // The sign of the rounded cents, so -0.004 is "$0.00", not "-$0.00".
    if int_value < 0 {
        format!("-${}.{:02}", formatted_dollars, cents)
    } else {
        format!("${}.{:02}", formatted_dollars, cents)
    }
}

#[cfg(test)]
mod tests {
    use super::format_currency;

    #[test]
    fn groups_thousands_and_rounds_to_cents() {
        assert_eq!(format_currency(1_234_567.891), "$1,234,567.89");
        assert_eq!(format_currency(999.995), "$1,000.00");
        assert_eq!(format_currency(0.0), "$0.00");
    }

    #[test]
    fn negatives_keep_their_sign() {
        assert_eq!(format_currency(-1500.5), "-$1,500.50");
        assert_eq!(format_currency(-0.25), "-$0.25");
    }

    #[test]
    fn a_value_that_rounds_to_zero_has_no_sign() {
        assert_eq!(format_currency(-0.004), "$0.00");
    }
}
