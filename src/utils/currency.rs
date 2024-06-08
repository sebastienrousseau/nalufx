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
        let mut count = 0;

        for digit in dollars_abs.chars().rev() {
            if count > 0 && count % 3 == 0 {
                result.push(',');
            }
            result.push(digit);
            count += 1;
        }

        result.chars().rev().collect::<String>()
    }

    let int_value = (value * 100.0).round() as i64; // Convert to integer cents
    let dollars = int_value / 100;
    let cents = (int_value % 100).abs(); // Absolute value for cents
    let formatted_dollars = format_dollars(dollars);

    if value < 0.0 {
        format!("-${}.{:02}", formatted_dollars, cents)
    } else {
        format!("${}.{:02}", formatted_dollars, cents)
    }
}
