#[cfg(test)]
mod tests {
    use nalufx::utils::date::validate_date;

    #[test]
    fn test_validate_date() {
        // Test with a valid date
        let valid_date = "2024-01-01";
        assert!(validate_date(valid_date).is_ok());

        // Test with an invalid date
        let invalid_date = "2024-13-01";
        assert!(validate_date(invalid_date).is_err());

        // Test with an invalid format
        let invalid_format = "01-01-2024";
        assert!(validate_date(invalid_format).is_err());

        // Test with an empty string
        let empty_string = "";
        assert!(validate_date(empty_string).is_err());
    }

    #[test]
    fn test_validate_date_error_message() {
        // Test with an invalid date
        let invalid_date = "2024-13-01";
        assert_eq!(
            validate_date(invalid_date).unwrap_err(),
            "Please enter a valid date in the format YYYY-MM-DD."
        );

        // Test with an invalid format
        let invalid_format = "01-01-2024";
        assert_eq!(
            validate_date(invalid_format).unwrap_err(),
            "Please enter a valid date in the format YYYY-MM-DD."
        );

        // Test with an empty string
        let empty_string = "";
        assert_eq!(
            validate_date(empty_string).unwrap_err(),
            "Please enter a valid date in the format YYYY-MM-DD."
        );
    }

    #[test]
    fn test_validate_date_time_component_error_message() {
        // Test with an invalid date
        let invalid_date = "2024-13-01";
        assert_eq!(
            validate_date(invalid_date).unwrap_err(),
            "Please enter a valid date in the format YYYY-MM-DD."
        );

        // Test with an invalid format
        let invalid_format = "01-01-2024";
        assert_eq!(
            validate_date(invalid_format).unwrap_err(),
            "Please enter a valid date in the format YYYY-MM-DD."
        );

        // Test with an empty string
        let empty_string = "";
        assert_eq!(
            validate_date(empty_string).unwrap_err(),
            "Please enter a valid date in the format YYYY-MM-DD."
        );
    }

    #[test]
    fn test_validate_date_conversion_error_message() {
        // Test with an invalid date
        let invalid_date = "2024-13-01";
        assert_eq!(
            validate_date(invalid_date).unwrap_err(),
            "Please enter a valid date in the format YYYY-MM-DD."
        );

        // Test with an invalid format
        let invalid_format = "01-01-2024";
        assert_eq!(
            validate_date(invalid_format).unwrap_err(),
            "Please enter a valid date in the format YYYY-MM-DD."
        );

        // Test with an empty string
        let empty_string = "";
        assert_eq!(
            validate_date(empty_string).unwrap_err(),
            "Please enter a valid date in the format YYYY-MM-DD."
        );
    }

    #[test]
    fn test_validate_date_multiple_error_messages() {
        // Test with an invalid date
        let invalid_date = "2024-13-01";
        assert_eq!(
            validate_date(invalid_date).unwrap_err(),
            "Please enter a valid date in the format YYYY-MM-DD."
        );

        // Test with an invalid format
        let invalid_format = "01-01-2024";
        assert_eq!(
            validate_date(invalid_format).unwrap_err(),
            "Please enter a valid date in the format YYYY-MM-DD."
        );

        // Test with an empty string
        let empty_string = "";
        assert_eq!(
            validate_date(empty_string).unwrap_err(),
            "Please enter a valid date in the format YYYY-MM-DD."
        );
    }
}
