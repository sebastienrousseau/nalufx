//! Forecasting and allocation.
//!
//! Every function here is deterministic: the same input gives the same
//! output, bit for bit, and anything that cannot be computed is an
//! [`AllocationError`](crate::errors::AllocationError) rather than a
//! substitute value.

use crate::errors::AllocationError;
use crate::models::forecast_dm::IntervalForecast;
use crate::{
    check_empty_inputs, check_input_lengths, check_invalid_data, check_outliers,
    fill_feature_matrix, normalize_features,
};
use augurs_core::{Fit, Predict};
use augurs_ets::AutoETS;
use ndarray::prelude::*;

pub use crate::utils::clustering::perform_clustering;

/// The probability every NaluFX prediction interval covers.
pub const FORECAST_LEVEL: f64 = 0.95;

/// The horizon, in days, of the allocations and forecasts NaluFX reports
/// when a caller does not choose one.
pub const DEFAULT_HORIZON_DAYS: usize = 6;

/// The largest absolute daily return accepted (a 100% move in one day).
const MAX_ABS_DAILY_RETURN: f64 = 1.0;

/// Allocates across the days of a forecast horizon by inverse volatility.
///
/// The daily returns are forecast `num_days` ahead with AutoETS, and each
/// day is weighted by the inverse of its 95% prediction interval's width,
/// which is proportional to the forecast's standard deviation. Days the
/// model is surer about get more; the weights are long-only and sum to 1.
///
/// This is the interim method until the M2 optimiser: it uses only the
/// uncertainty of the forecast, not its expected return.
///
/// # Arguments
///
/// * `daily_returns` - Historical daily returns, oldest first.
/// * `num_days` - The number of days to allocate across.
///
/// # Returns
///
/// One weight per day, each at least 0, summing to 1.
///
/// # Errors
///
/// - [`AllocationError::EmptyInput`] if there are no returns.
/// - [`AllocationError::InvalidData`] if a return is NaN or infinite.
/// - [`AllocationError::OutlierData`] if a return exceeds 100% in absolute value.
/// - [`AllocationError::InvalidHorizon`] if `num_days` is 0.
/// - [`AllocationError::ForecastingError`] if AutoETS cannot fit the series.
/// - [`AllocationError::ZeroVolatility`] if a forecast interval has no width beyond rounding.
///
/// # Examples
///
/// ```
/// use nalufx::utils::calculations::calculate_optimal_allocation;
///
/// let daily_returns = [0.01, -0.02, 0.015, 0.003, -0.007, 0.012, -0.004, 0.009, -0.011, 0.006];
/// let weights = calculate_optimal_allocation(&daily_returns, 3).unwrap();
/// assert_eq!(weights.len(), 3);
/// assert!((weights.iter().sum::<f64>() - 1.0).abs() < 1e-9);
/// ```
pub fn calculate_optimal_allocation(
    daily_returns: &[f64],
    num_days: usize,
) -> Result<Vec<f64>, AllocationError> {
    validate_series(daily_returns, num_days)?;
    check_outliers!(MAX_ABS_DAILY_RETURN, daily_returns)?;
    // A constant series has no volatility to weight by. Its forecast
    // interval is rounding noise, which around zero is too small (about
    // 1e-98) for any floor relative to the interval itself to catch.
    if is_constant(daily_returns) {
        return Err(AllocationError::ZeroVolatility);
    }
    let forecast = forecast_time_series(daily_returns, num_days)?;
    inverse_volatility(&resolved_widths(&forecast))
}

/// Whether every value equals the first to within rounding.
fn is_constant(data: &[f64]) -> bool {
    let (lo, hi) =
        data.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &x| (lo.min(x), hi.max(x)));
    hi - lo <= NOISE_ULPS * f64::EPSILON * lo.abs().max(hi.abs())
}

/// Spreads up to this many ulps of a value's magnitude are rounding noise.
const NOISE_ULPS: f64 = 64.0;

/// The interval widths, with any width at the floating-point noise floor
/// set to exactly 0.
///
/// A constant series fits with an interval a few ulps wide (about 1e-17
/// on returns of 0.01): rounding, not uncertainty. Weighting by its
/// inverse would turn that noise into an allocation, so it counts as
/// zero volatility.
fn resolved_widths(forecast: &IntervalForecast) -> Vec<f64> {
    forecast
        .widths()
        .into_iter()
        .zip(forecast.lower.iter().zip(&forecast.upper))
        .map(|(width, (lo, hi))| {
            let floor = NOISE_ULPS * f64::EPSILON * lo.abs().max(hi.abs()).max(f64::MIN_POSITIVE);
            if width <= floor {
                0.0
            } else {
                width
            }
        })
        .collect()
}

/// Weights proportional to the inverse of each volatility, summing to 1.
///
/// # Arguments
///
/// * `volatilities` - One volatility (or any quantity proportional to it) per item.
///
/// # Errors
///
/// - [`AllocationError::EmptyInput`] if there are no volatilities.
/// - [`AllocationError::ZeroVolatility`] if one is 0, or so small its inverse overflows.
/// - [`AllocationError::InvalidData`] if one is negative, NaN or infinite.
///
/// # Examples
///
/// ```
/// use nalufx::utils::calculations::inverse_volatility;
///
/// assert_eq!(inverse_volatility(&[1.0, 1.0]).unwrap(), vec![0.5, 0.5]);
/// assert!(inverse_volatility(&[1.0, 1e-310]).is_err());
/// let weights = inverse_volatility(&[1.0, 3.0]).unwrap();
/// assert!((weights[0] - 0.75).abs() < 1e-12);
/// ```
pub fn inverse_volatility(volatilities: &[f64]) -> Result<Vec<f64>, AllocationError> {
    if volatilities.is_empty() {
        return Err(AllocationError::EmptyInput);
    }
    if volatilities.iter().any(|v| !v.is_finite() || *v < 0.0) {
        return Err(AllocationError::InvalidData);
    }
    if volatilities.contains(&0.0) {
        return Err(AllocationError::ZeroVolatility);
    }
    let inverse: Vec<f64> = volatilities.iter().map(|v| 1.0 / v).collect();
    let total: f64 = inverse.iter().sum();
    // A volatility so small (subnormal) that its inverse overflows is zero
    // for any practical purpose; dividing by an infinite total would give 0
    // or NaN weights.
    if !total.is_finite() {
        return Err(AllocationError::ZeroVolatility);
    }
    Ok(inverse.into_iter().map(|w| w / total).collect())
}

/// Extracts features from the input data for clustering.
///
/// The four series become the four columns of a feature matrix, each
/// normalised to zero mean and unit variance.
///
/// # Arguments
///
/// * `daily_returns` - A slice of daily returns.
/// * `cash_flows` - A slice of cash flows.
/// * `market_indices` - A slice of market indices.
/// * `fund_characteristics` - A slice of fund characteristics.
///
/// # Returns
///
/// A feature matrix (`Array2<f64>`) with one row per observation.
///
/// # Errors
///
/// Returns an error if the slices differ in length, are empty, hold NaN or
/// infinite values, or hold outliers.
///
/// # Examples
///
/// ```
/// use nalufx::extract_features;
/// use nalufx::errors::AllocationError;
///
/// let daily_returns = vec![0.01, 0.02, -0.01];
/// let cash_flows = vec![1000.0, 1020.0, 1010.0];
/// let market_indices = vec![1.0, 1.01, 1.02];
/// let fund_characteristics = vec![0.5, 0.6, 0.7];
/// let features = extract_features!(&daily_returns, &cash_flows, &market_indices, &fund_characteristics).unwrap();
/// assert_eq!(features.shape(), &[3, 4]);
/// # Ok::<(), AllocationError>(())
/// ```
pub fn extract_features(
    daily_returns: &[f64],
    cash_flows: &[f64],
    market_indices: &[f64],
    fund_characteristics: &[f64],
) -> Result<Array2<f64>, AllocationError> {
    check_input_lengths!(daily_returns, cash_flows, market_indices, fund_characteristics)?;
    check_empty_inputs!(daily_returns, cash_flows, market_indices, fund_characteristics)?;
    check_invalid_data!(daily_returns, cash_flows)?;
    check_outliers!(MAX_ABS_DAILY_RETURN, daily_returns)?;
    check_outliers!(1_000_000.0, cash_flows)?;

    let n = daily_returns.len();
    let mut features = Array2::<f64>::zeros((n, 4));
    fill_feature_matrix!(
        features,
        n,
        daily_returns,
        cash_flows,
        market_indices,
        fund_characteristics
    );
    normalize_features!(features);
    Ok(features)
}

/// Forecasts a time series with AutoETS, with its 95% prediction interval.
///
/// AutoETS picks the best non-seasonal exponential smoothing model by
/// AICc. The search is deterministic: it leaves out multiplicative-trend
/// models, the only ones whose intervals augurs simulates at random.
///
/// # Arguments
///
/// * `data` - Historical values, oldest first.
/// * `num_days` - The number of days to forecast.
///
/// # Returns
///
/// The point forecast with its interval at [`FORECAST_LEVEL`].
///
/// # Errors
///
/// - [`AllocationError::EmptyInput`] if there is no data.
/// - [`AllocationError::InvalidData`] if a value is NaN or infinite.
/// - [`AllocationError::InvalidHorizon`] if `num_days` is 0.
/// - [`AllocationError::ForecastingError`] if no model fits, or the fit gives no interval.
///
/// # Examples
///
/// ```
/// use nalufx::utils::calculations::forecast_time_series;
///
/// let data = [100.0, 101.0, 102.0, 101.5, 103.0, 102.2, 104.1, 103.6];
/// let forecast = forecast_time_series(&data, 3).unwrap();
/// assert_eq!(forecast.point.len(), 3);
/// assert!(forecast.lower[0] <= forecast.point[0] && forecast.point[0] <= forecast.upper[0]);
/// ```
pub fn forecast_time_series(
    data: &[f64],
    num_days: usize,
) -> Result<IntervalForecast, AllocationError> {
    validate_series(data, num_days)?;
    let model = AutoETS::new(1, "ZZN").map_err(failed)?.fit(data).map_err(failed)?;
    to_interval_forecast(model.predict(num_days, FORECAST_LEVEL).map_err(failed)?)
}

/// An augurs forecast as an [`IntervalForecast`], checked to have an
/// interval and to be finite throughout.
fn to_interval_forecast(
    forecast: augurs_core::Forecast,
) -> Result<IntervalForecast, AllocationError> {
    let intervals =
        forecast.intervals.ok_or_else(|| failed("the model gave no prediction interval"))?;
    let result = IntervalForecast {
        level: intervals.level,
        point: forecast.point,
        lower: intervals.lower,
        upper: intervals.upper,
    };
    let finite = |v: &[f64]| v.iter().all(|x| x.is_finite());
    if !(finite(&result.point) && finite(&result.lower) && finite(&result.upper)) {
        return Err(failed("the forecast is not finite"));
    }
    Ok(result)
}

/// Wraps a forecasting failure.
fn failed(err: impl std::fmt::Display) -> AllocationError {
    AllocationError::ForecastingError(err.to_string())
}

/// Checks a series and a horizon before anything is computed from them.
fn validate_series(data: &[f64], num_days: usize) -> Result<(), AllocationError> {
    check_empty_inputs!(data)?;
    check_invalid_data!(data)?;
    if num_days == 0 {
        return Err(AllocationError::InvalidHorizon);
    }
    Ok(())
}
