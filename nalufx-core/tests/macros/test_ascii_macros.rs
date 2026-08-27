#[cfg(test)]
mod tests {
    use nalufx::macro_ascii;

    #[test]
    fn test_macro_ascii_success() {
        let art = macro_ascii!("Hi");
        // The standard font applies its default smushing, so adjacent
        // glyphs share a column. The previous expectation encoded the
        // unsmushed layout and never ran, so it was never corrected.
        assert_eq!(
            art,
            " _   _ _ \n| | | (_)\n| |_| | |\n|  _  | |\n|_| |_|_|\n         \n"
        );
    }

    #[test]
    #[should_panic(expected = "Failed to generate ASCII art: Failed to convert text to ASCII art")]
    fn test_macro_ascii_empty_input() {
        let _art = macro_ascii!("");
    }

    #[test]
    #[should_panic(expected = "Failed to generate ASCII art: Failed to convert text to ASCII art")]
    fn test_macro_ascii_invalid_input() {
        let _art = macro_ascii!("🦀");
    }
}
