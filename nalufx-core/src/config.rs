use std::env;

/// Represents the configuration for the application.
///
/// This struct holds the configuration values required to run the application,
/// such as the server address. The configuration values are typically loaded
/// from environment variables.
///
/// # Fields
///
/// * `server_addr` - A string containing the address of the server.
///
/// # Examples
///
/// ```
/// use std::env;
/// use nalufx::config::Config;
///
/// // Set the environment variable for demonstration purposes
/// env::set_var("SERVER_ADDR", "127.0.0.1:8080");
///
/// let config = Config::from_env().expect("Failed to load configuration");
/// assert_eq!(config.server_addr, "127.0.0.1:8080");
/// println!("Server address: {}", config.server_addr);
///
/// // Unset the environment variable to avoid side effects
/// env::remove_var("SERVER_ADDR");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Config {
    /// A string containing the address of the server.
    pub server_addr: String,
}

impl Config {
    /// Creates a new `Config` instance by loading values from environment variables.
    ///
    /// This function attempts to read the required configuration values from environment variables.
    /// If any of the required environment variables are not set, it returns an error.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `Config` instance if successful, or an `env::VarError` if any
    /// environment variable is not set.
    ///
    /// # Errors
    ///
    /// Returns an error if the `SERVER_ADDR` environment variable is not set.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::env;
    /// use nalufx::config::Config;
    ///
    /// // Set the environment variable for demonstration purposes
    /// env::set_var("SERVER_ADDR", "127.0.0.1:8080");
    ///
    /// let config = Config::from_env().expect("Failed to load configuration");
    /// assert_eq!(config.server_addr, "127.0.0.1:8080");
    /// println!("Server address: {}", config.server_addr);
    ///
    /// // Unset the environment variable to avoid side effects
    /// env::remove_var("SERVER_ADDR");
    /// ```
    pub fn from_env() -> Result<Self, env::VarError> {
        let server_addr = env::var("SERVER_ADDR")?;
        Ok(Self { server_addr })
    }
}
