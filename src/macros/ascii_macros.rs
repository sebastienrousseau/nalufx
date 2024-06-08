/// A macro for generating ASCII art from text.
///
/// This macro takes a string literal as input and generates ASCII art using the `generate_ascii_art` function.
/// If the conversion is successful, the macro returns the ASCII art as a string.
/// If an error occurs during the conversion, the macro panics with an error message.
///
/// # Examples
///
/// ```
/// use nalufx::macro_ascii;
///
/// let art = macro_ascii!("Hello, world!");
/// println!("{}", art);
/// ```
#[macro_export]
macro_rules! macro_ascii {
    ($text:expr) => {
        match $crate::utils::ascii::generate_ascii_art($text) {
            Ok(art) => art,
            Err(err) => panic!("Failed to generate ASCII art: {}", err),
        }
    };
}
