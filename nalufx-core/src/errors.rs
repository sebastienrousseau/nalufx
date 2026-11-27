use thiserror::Error;

/// Represents an error that can occur in the NaluFx library.
///
/// This enum encapsulates various types of errors that might occur while using the NaluFx library.
/// It provides detailed error messages to help diagnose issues during the execution of various
/// functions within the library.
///
/// # Variants
///
/// * `ClusteringError(String)` - An error occurred during clustering.
/// * `ForecastingError(String)` - An error occurred during time series forecasting.
/// * `InputMismatch` - The input slices must have the same length.
/// * `EmptyInput` - The input slices cannot be empty.
/// * `InvalidData` - The input data contains missing or invalid values.
/// * `InputError(std::io::Error)` - An error occurred while reading user input.
/// * `InvalidOption` - An invalid option was selected.
/// * `OutlierData` - The input data contains outliers.
/// * `TechnicalAnalysisError(String)` - An error occurred during technical analysis.
/// * `PortfolioOptimizationError(String)` - An error occurred during portfolio optimization.
///
/// # Examples
///
/// ```
/// use nalufx::errors::NaluFxError;
///
/// fn example_function() -> Result<(), NaluFxError> {
///     Err(NaluFxError::InputMismatch)
/// }
#[derive(Debug, Error)]
pub enum NaluFxError {
    /// An error occurred during clustering.
    #[error("Error during clustering: {0}")]
    ClusteringError(String),

    /// An error occurred during time series forecasting.
    #[error("Error during time series forecasting: {0}")]
    ForecastingError(String),

    /// The input slices must have the same length.
    #[error("Input slices must have the same length")]
    InputMismatch,

    /// The input slices cannot be empty.
    #[error("Input slices cannot be empty")]
    EmptyInput,

    /// The input data contains missing or invalid values.
    #[error("Input data contains missing or invalid values")]
    InvalidData,

    /// An error occurred while reading user input.
    #[error("Error reading user input: {0}")]
    InputError(#[from] std::io::Error),

    /// An invalid option was selected.
    #[error("Invalid option. Please enter a number between 1 and 4.")]
    InvalidOption,

    /// The input data contains outliers.
    #[error("Input data contains outliers")]
    OutlierData,

    /// An error occurred during technical analysis.
    #[error("Error during technical analysis: {0}")]
    TechnicalAnalysisError(String),

    /// An error occurred during portfolio optimization.
    #[error("Error during portfolio optimization: {0}")]
    PortfolioOptimizationError(String),

    /// An error occurred during stock analysis.
    #[error("Error during stock analysis: {0}")]
    StockAnalysisError(String),

    /// An error occurred with an HTTP request.
    #[error("HTTP request error: {0}")]
    HttpRequestError(#[from] reqwest::Error),

    /// Not all assets have the same number of returns
    #[error("Not all assets have the same number of returns")]
    UnequalReturnsLength,

    /// Flattened returns vector length does not match expected shape
    #[error("Flattened returns vector length does not match expected shape")]
    InvalidReturnsShape,

    /// An error occurred while parsing a date.
    #[error("Failed to parse date")]
    DateParseError(#[from] time::error::Parse),

    /// An error occurred with CSV parsing.
    #[error("CSV parsing error: {0}")]
    CsvError(#[from] csv::Error),

    /// An error occurred during JSON serialization/deserialization.
    #[error("JSON serialization/deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// An error occurred in the NaluFx library.
    #[error("NaluFxError: {0}")]
    NaluFxError(String),

    /// An error occurred with string manipulation.
    #[error("String error: {0}")]
    StringError(String),
}

/// Why a forecast, an estimate or an allocation could not be computed;
/// defined in [`nalufx_engine`] and re-exported here.
pub use nalufx_engine::AllocationError;
