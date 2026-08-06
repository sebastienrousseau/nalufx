use std::error::Error;
use std::fmt;

/// Error type for ASCII art generation failures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AsciiArtError {
    /// Represents a failure to load the FIGlet.
    FontLoadError,
    /// Represents a failure to convert text to ASCII art.
    ConversionError,
}

impl fmt::Display for AsciiArtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::FontLoadError => write!(f, "Failed to load FIGlet"),
            Self::ConversionError => {
                write!(f, "Failed to convert text to ASCII art")
            },
        }
    }
}

impl Error for AsciiArtError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
