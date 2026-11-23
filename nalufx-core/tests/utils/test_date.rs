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

/// Characterisation of `validate_date`: which inputs it accepts and the
/// instant each one maps to (UTC midnight, as a Unix timestamp).
///
/// The table was recorded against the chrono implementation and is kept
/// unchanged across the move to the `time` crate, so it pins the parsing
/// rules (lenient padding, leading whitespace, explicit year signs,
/// calendar validation, year range) rather than any one library.
#[cfg(test)]
mod characterisation {
    use nalufx::utils::date::validate_date;

    /// The single line that names the date library's accessor.
    fn unix(input: &str) -> Option<i64> {
        validate_date(input).ok().map(|d| d.timestamp())
    }

    const CASES: &[(&str, Option<i64>)] = &[
        ("2024-01-01", Some(1_704_067_200)),
        ("1970-01-01", Some(0)),
        ("1969-12-31", Some(-86_400)),
        ("2024-02-29", Some(1_709_164_800)),
        ("2023-02-29", None),
        ("2024-04-31", None),
        ("2024-00-10", None),
        ("2024-01-00", None),
        ("2024-13-01", None),
        ("2024-1-1", Some(1_704_067_200)),
        ("2024-01-1", Some(1_704_067_200)),
        (" 2024-01-01", Some(1_704_067_200)),
        ("\t2024-01-01", Some(1_704_067_200)),
        ("\u{a0}2024-01-01", Some(1_704_067_200)),
        ("2024- 01- 01", Some(1_704_067_200)),
        ("2024 -01-01", None),
        ("2024-01-01 ", None),
        ("2024-01-01T00:00:00", None),
        ("2024-001-01", None),
        ("2024-01-001", None),
        ("20240-01-01", None),
        ("99-01-01", Some(-59_042_995_200)),
        ("0-01-01", Some(-62_167_219_200)),
        ("+2024-01-01", Some(1_704_067_200)),
        ("+02024-01-01", Some(1_704_067_200)),
        ("-0001-01-01", Some(-62_198_755_200)),
        ("+10000-01-01", Some(253_402_300_800)),
        ("+262142-12-31", Some(8_210_266_790_400)),
        ("+262143-01-01", None),
        ("-262143-01-01", Some(-8_334_601_228_800)),
        ("-262144-12-31", None),
        ("+99999999999999999999-01-01", None),
        ("+-2024-01-01", None),
        ("2024/01/01", None),
        ("01-01-2024", None),
        ("", None),
        ("-", None),
        ("2024", None),
        ("2024-01", None),
        ("2024-01-", None),
        ("２０２４-01-01", None),
    ];

    #[test]
    fn accepts_and_maps_exactly_these_inputs() {
        for &(input, expected) in CASES {
            assert_eq!(unix(input), expected, "input {input:?}");
        }
    }

    #[test]
    fn every_rejection_has_the_same_message() {
        for &(input, expected) in CASES {
            if expected.is_none() {
                assert_eq!(
                    validate_date(input).unwrap_err(),
                    "Please enter a valid date in the format YYYY-MM-DD.",
                    "input {input:?}"
                );
            }
        }
    }
}
