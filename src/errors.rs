use thiserror::Error;

/// Represents an error that can occur in the NaluFx library.
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
}

/// Represents an error that can occur during allocation.
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
