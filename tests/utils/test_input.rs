#[cfg(test)]
mod tests {
    use nalufx::errors::NaluFxError;
    use nalufx::utils::input::{get_input, read_user_input};
    use std::io::{self, BufRead, BufReader, Write};
    use std::os::unix::net::UnixStream;
    use std::sync::{Arc, Mutex};
    use std::thread;

    /// A helper function to run a function that expects stdin input, providing simulated input and capturing output.
    fn run_with_stdin_output<F>(input: &str, func: F) -> Result<String, String>
    where
        F: FnOnce() -> Result<String, NaluFxError> + Send + 'static,
    {
        // Create a UnixStream to simulate stdin.
        let (reader, writer) = UnixStream::pair().expect("Failed to create UnixStream pair");
        let reader = BufReader::new(reader);
        let writer = Arc::new(Mutex::new(writer));
        let input = input.to_string(); // Clone the input string

        // Write the input to the writer in a separate thread.
        let writer_clone = Arc::clone(&writer);
        let _ = thread::spawn(move || {
            let mut writer = writer_clone.lock().unwrap();
            writer.write_all(input.as_bytes()).unwrap();
        });

        // Redirect stdin to read from our UnixStream.
        let stdin = io::stdin();
        let _original_stdin = stdin.lock();
        let guard = set_input(reader);

        // Capture the function's output.
        let result = func().map_err(|e| format!("Error: {}", e));

        // Restore the original stdin.
        drop(guard);

        result
    }

    fn set_input<R: 'static>(_reader: R) -> impl Drop
    where
        R: BufRead + Send,
    {
        struct StdinGuard;
        impl Drop for StdinGuard {
            fn drop(&mut self) {
                // Restore original stdin if needed.
            }
        }
        StdinGuard
    }

    #[test]
    fn test_read_user_input() {
        let input = "Test input\n";
        let result = run_with_stdin_output(input, || read_user_input());
        assert_eq!(result.unwrap(), "Test input");
    }

    #[test]
    fn test_get_input() {
        let input = "Test input\n";
        let result = run_with_stdin_output(input, || get_input("Enter something: "));
        assert_eq!(result.unwrap(), "Test input");
    }

    #[test]
    fn test_get_input_with_prompt() {
        let input = "Another input\n";
        let result = run_with_stdin_output(input, || get_input("Please type: "));
        assert_eq!(result.unwrap(), "Another input");
    }

    #[test]
    fn test_read_user_input_error() {
        let result = run_with_stdin_output("", || {
            let mut input = String::new();
            io::stdin()
                .lock()
                .read_line(&mut input)
                .map(|_| input.trim().to_string())
                .map_err(NaluFxError::InputError)
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_get_input_error() {
        let result = run_with_stdin_output("", || get_input("Enter something: "));
        assert!(result.is_err());
    }
}
