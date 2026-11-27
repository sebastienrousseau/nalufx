//! API keys: kept out of debug prints and logs, and wiped from memory
//! when dropped.

use secrecy::{ExposeSecret, SecretString};
use std::fmt;

/// A provider's API key. `{:?}` prints `[REDACTED]`; there is no
/// `Display`, so a key cannot be formatted into a message by mistake.
/// Only the adapters read it, to set the request header.
#[derive(Clone)]
pub struct ApiKey(SecretString);

impl ApiKey {
    /// Wraps a key.
    #[must_use]
    pub fn new(key: impl Into<String>) -> Self {
        Self(SecretString::from(key.into()))
    }

    /// The key itself, for a request header.
    pub(crate) fn expose(&self) -> &str {
        self.0.expose_secret()
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl From<String> for ApiKey {
    fn from(key: String) -> Self {
        Self::new(key)
    }
}

impl From<&str> for ApiKey {
    fn from(key: &str) -> Self {
        Self::new(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_key_prints_as_redacted_and_reads_back_whole() {
        let key = ApiKey::from("sk-secret");
        assert_eq!(format!("{key:?}"), "[REDACTED]");
        assert_eq!(key.expose(), "sk-secret");
    }
}
