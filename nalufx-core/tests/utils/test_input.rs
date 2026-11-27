#[cfg(test)]
mod tests {
    use nalufx::errors::NaluFxError;
    use nalufx::utils::input::read_line_from;
    use std::io::{BufRead, BufReader};

    /// The production reader, trimmed as `get_input` trims it.
    fn read_input_for_test<R: BufRead>(reader: R) -> Result<String, NaluFxError> {
        Ok(read_line_from(reader)?.trim().to_string())
    }

    #[test]
    fn test_read_user_input() {
        let input = "Hello, World!";
        let expected = "Hello, World!";
        let reader = BufReader::new(input.as_bytes());
        let result = read_input_for_test(reader).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_input() {
        let input = "Hello, World!\n";
        let expected = "Hello, World!";
        let reader = BufReader::new(input.as_bytes());
        let result = read_input_for_test(reader).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        let expected = "";
        let reader = BufReader::new(input.as_bytes());
        let result = read_input_for_test(reader).unwrap();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_input_with_whitespace() {
        let input = "  Hello, World!  \n";
        let expected = "Hello, World!";
        let reader = BufReader::new(input.as_bytes());
        let result = read_input_for_test(reader).unwrap();
        assert_eq!(result, expected);
    }

    /// A reader that always fails.
    struct Broken;

    impl std::io::Read for Broken {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("disk on fire"))
        }
    }

    #[test]
    fn the_line_ending_is_kept_and_a_read_error_is_typed() {
        assert_eq!(read_line_from("a line\n".as_bytes()).unwrap(), "a line\n");
        let err = read_line_from(BufReader::new(Broken)).unwrap_err();
        assert!(matches!(err, NaluFxError::InputError(_)), "{err}");
    }
}
