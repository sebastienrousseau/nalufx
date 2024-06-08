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
/// * `SentimentAnalysisError(String)` - An error occurred during sentiment analysis.
/// * `ReinforcementLearningError(String)` - An error occurred during reinforcement learning.
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
/// ```
#[derive(Debug, Error)]
pub enum NaluFxError {
    /// An error occurred during clustering.
    #[error("Error during clustering: {0}")]
    ClusteringError(String),

    /// An error occurred during time series forecasting.
    #[error("Error during time series forecasting: {0}")]
    ForecastingError(String),

    /// An error occurred during sentiment analysis.
    #[error("Error during sentiment analysis: {0}")]
    SentimentAnalysisError(String),

    /// An error occurred during reinforcement learning.
    #[error("Error during reinforcement learning: {0}")]
    ReinforcementLearningError(String),

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
    DateParseError(#[from] chrono::ParseError),

    /// An error occurred in the NaluFx library.
    #[error("NaluFxError: {0}")]
    NaluFxError(String),
}

/// Represents an error that can occur during allocation.
///
/// This enum encapsulates various types of errors that might occur during the allocation process.
/// It provides detailed error messages to help diagnose issues during the execution of allocation
/// functions within the library.
///
/// # Variants
///
/// * `InputMismatch` - The input slices must have the same length.
/// * `EmptyInput` - The input slices cannot be empty.
/// * `ClusteringError(String)` - An error occurred during clustering.
/// * `InvalidData` - The input data contains missing or invalid values.
/// * `OutlierData` - The input data contains outliers.
/// * `ForecastingError(String)` - An error occurred during time series forecasting.
/// * `SentimentAnalysisError(String)` - An error occurred during sentiment analysis.
/// * `ReinforcementLearningError(String)` - An error occurred during reinforcement learning.
///
/// # Examples
///
/// ```
/// use nalufx::errors::AllocationError;
///
/// fn example_function() -> Result<(), AllocationError> {
///     Err(AllocationError::EmptyInput)
/// }
/// ```
#[derive(Debug, Error, PartialEq)]
pub enum AllocationError {
    /// The input slices must have the same length.
    #[error("Input slices must have the same length")]
    InputMismatch,

    /// The input slices cannot be empty.
    #[error("Input slices cannot be empty")]
    EmptyInput,

    /// An error occurred during clustering.
    #[error("Error during clustering: {0}")]
    ClusteringError(String),

    /// The input data contains missing or invalid values.
    #[error("Input data contains missing or invalid values")]
    InvalidData,

    /// The input data contains outliers.
    #[error("Input data contains outliers")]
    OutlierData,

    /// An error occurred during time series forecasting.
    #[error("Error during time series forecasting: {0}")]
    ForecastingError(String),

    /// An error occurred during sentiment analysis.
    #[error("Error during sentiment analysis: {0}")]
    SentimentAnalysisError(String),

    /// An error occurred during reinforcement learning.
    #[error("Error during reinforcement learning: {0}")]
    ReinforcementLearningError(String),
}
