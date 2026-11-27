//! The one error type every engine function returns.

use thiserror::Error;

/// Why a forecast, an estimate or an allocation could not be computed.
///
/// Every engine function returns this rather than a substitute value:
/// there is no fallback number anywhere in the engine.
///
/// # Examples
///
/// ```
/// use nalufx_engine::AllocationError;
///
/// fn example_function() -> Result<(), AllocationError> {
///     Err(AllocationError::EmptyInput)
/// }
/// assert_eq!(example_function().unwrap_err().to_string(), "Input slices cannot be empty");
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

    /// The forecast horizon is zero days.
    #[error("The forecast horizon must be at least one day")]
    InvalidHorizon,

    /// A series or a forecast has no variation to weight by, so an
    /// inverse-volatility weight does not exist.
    #[error("The forecast has zero volatility, so no inverse-volatility weight exists")]
    ZeroVolatility,

    /// An error occurred with a price used as a divisor.
    #[error("A start price of {0} cannot be used to compute a return")]
    InvalidPrice(f64),

    /// No weights satisfy the constraints.
    #[error("The constraints cannot be satisfied: {0}")]
    Infeasible(String),

    /// The optimiser stopped without a solution.
    #[error("The optimiser did not converge: {0}")]
    SolverFailed(String),

    /// No allocation method has this name.
    #[error("Unknown allocation method `{0}`")]
    UnknownMethod(String),

    /// A method or estimator parameter is out of range.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}
