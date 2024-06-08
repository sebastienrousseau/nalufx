#[cfg(test)]
mod tests {
    use nalufx::errors::NaluFxError;
    use std::io::{BufRead, BufReader};

    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn read_input_for_test<R: BufRead>(mut reader: R) -> Result<String, NaluFxError> {
        let mut input = String::new();
        let _ = reader
            .read_line(&mut input)
            .map_err(NaluFxError::InputError)?;
        Ok(input.trim().to_string())
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn get_input_for_test_err<R: BufRead>(mut reader: R) -> Result<String, NaluFxError> {
        let mut input = String::new();
        let _ = reader
            .read_line(&mut input)
            .map_err(NaluFxError::InputError)?;
        Ok(input.trim().to_string())
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
        let result = get_input_for_test_err(reader).unwrap();
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
        let result = get_input_for_test_err(reader).unwrap();
        assert_eq!(result, expected);
    }
}
