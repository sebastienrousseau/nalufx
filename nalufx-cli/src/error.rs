//! What can go wrong, the exit code for each, and what to do about it.

use nalufx_engine::AllocationError;
use nalufx_llms::error::LlmError;
use std::fmt;

/// A failed command. Each kind has its own exit code, so scripts can tell
/// a typo from a network failure.
///
/// | Code | Kind |
/// | --- | --- |
/// | 0 | success |
/// | 1 | anything else |
/// | 2 | a usage error: an unknown flag or a bad value (from clap) |
/// | 3 | [`CliError::Input`]: the data could not be read or used |
/// | 4 | [`CliError::NoAnswer`]: valid data, but the maths has no answer |
/// | 5 | [`CliError::Provider`]: the network, a price feed or a language model failed |
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    /// The data could not be read or used.
    Input {
        /// What went wrong.
        message: String,
        /// What to do about it.
        hint: Option<String>,
    },
    /// The data is valid but the maths has no answer for it.
    NoAnswer {
        /// What went wrong.
        message: String,
        /// What to do about it.
        hint: Option<String>,
    },
    /// The network, a price feed or a language model failed.
    Provider {
        /// What went wrong.
        message: String,
        /// What to do about it.
        hint: Option<String>,
    },
}

impl CliError {
    /// An input error with a hint.
    pub fn input(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self::Input { message: message.into(), hint: Some(hint.into()) }
    }

    /// The process exit code for this error.
    #[must_use]
    pub const fn exit_code(&self) -> u8 {
        match self {
            Self::Input { .. } => 3,
            Self::NoAnswer { .. } => 4,
            Self::Provider { .. } => 5,
        }
    }

    /// What to do about it, if there is something.
    #[must_use]
    pub fn hint(&self) -> Option<&str> {
        match self {
            Self::Input { hint, .. }
            | Self::NoAnswer { hint, .. }
            | Self::Provider { hint, .. } => hint.as_deref(),
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (Self::Input { message, .. }
        | Self::NoAnswer { message, .. }
        | Self::Provider { message, .. }) = self;
        f.write_str(message)
    }
}

impl std::error::Error for CliError {}

impl From<AllocationError> for CliError {
    fn from(err: AllocationError) -> Self {
        let message = err.to_string();
        match err {
            AllocationError::Infeasible(_) => Self::NoAnswer {
                message,
                hint: Some("raise --max-weight or lower --min-weight: n assets need n x max-weight >= 1 and n x min-weight <= 1".into()),
            },
            AllocationError::ZeroVolatility => Self::NoAnswer {
                message,
                hint: Some("a series with no variation cannot be weighted; check the data".into()),
            },
            AllocationError::ClusteringError(_) | AllocationError::ForecastingError(_) | AllocationError::SolverFailed(_) => {
                Self::NoAnswer { message, hint: None }
            },
            AllocationError::UnknownMethod(_) => {
                Self::Input { message, hint: Some("run `nalufx methods` to see the methods".into()) }
            },
            AllocationError::InputMismatch
            | AllocationError::EmptyInput
            | AllocationError::InvalidData
            | AllocationError::OutlierData
            | AllocationError::InvalidHorizon
            | AllocationError::InvalidPrice(_)
            | AllocationError::InvalidParameter(_) => Self::Input { message, hint: None },
        }
    }
}

impl From<LlmError> for CliError {
    fn from(err: LlmError) -> Self {
        let message = err.to_string();
        match err {
            LlmError::MissingApiKey { variable, .. } => Self::Input {
                message,
                hint: Some(format!("export {variable}=<your key>, or put it in a .env file")),
            },
            LlmError::Config(_) => Self::Input {
                message,
                hint: Some("run `nalufx providers` to see the providers".into()),
            },
            LlmError::Schema { .. } => Self::Provider {
                message,
                hint: Some(
                    "the model replied off the report schema; retry, or try a larger model".into(),
                ),
            },
            LlmError::Transport { .. }
            | LlmError::Status { .. }
            | LlmError::ParseResponse { .. } => Self::Provider { message, hint: None },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_kind_has_its_exit_code() {
        assert_eq!(CliError::input("x", "y").exit_code(), 3);
        assert_eq!(CliError::from(AllocationError::Infeasible("x".into())).exit_code(), 4);
        let timeout = LlmError::Transport { provider: "p", message: String::new(), timeout: true };
        assert_eq!(CliError::from(timeout).exit_code(), 5);
    }

    #[test]
    fn infeasible_bounds_and_a_missing_key_say_what_to_do() {
        assert!(CliError::from(AllocationError::Infeasible("x".into()))
            .hint()
            .unwrap()
            .contains("--max-weight"));
        let key = CliError::from(LlmError::MissingApiKey {
            provider: "anthropic",
            variable: "ANTHROPIC_API_KEY",
        });
        assert_eq!(key.exit_code(), 3);
        assert!(key.hint().unwrap().contains("ANTHROPIC_API_KEY"));
    }
}
