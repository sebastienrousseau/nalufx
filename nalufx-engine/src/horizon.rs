//! The interim allocator: one series, spread over the days of a horizon.
//!
//! This weights the days of a single series' forecast by inverse
//! volatility. To allocate across assets, use [`crate::allocate`].

use crate::forecast::{forecast_time_series, validate_series, IntervalForecast};
use crate::{check_outliers, AllocationError};

/// The horizon, in days, of the allocations and forecasts NaluFX reports
/// when a caller does not choose one.
pub const DEFAULT_HORIZON_DAYS: usize = 6;

/// The largest absolute daily return accepted (a 100% move in one day).
pub(crate) const MAX_ABS_DAILY_RETURN: f64 = 1.0;

/// Allocates across the days of a forecast horizon by inverse volatility.
///
/// The daily returns are forecast `num_days` ahead with AutoETS, and each
/// day is weighted by the inverse of its 95% prediction interval's width,
/// which is proportional to the forecast's standard deviation. Days the
/// model is surer about get more; the weights are long-only and sum to 1.
///
/// It uses only the uncertainty of one series' forecast, not its expected
/// return; for daily returns that look like noise the weights come out
/// close to equal. To allocate across assets, use [`crate::allocate`].
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
/// use nalufx_engine::horizon::allocate_over_horizon;
///
/// let daily_returns = [0.01, -0.02, 0.015, 0.003, -0.007, 0.012, -0.004, 0.009, -0.011, 0.006];
/// let weights = allocate_over_horizon(&daily_returns, 3).unwrap();
/// assert_eq!(weights.len(), 3);
/// assert!((weights.iter().sum::<f64>() - 1.0).abs() < 1e-9);
/// ```
pub fn allocate_over_horizon(
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
/// use nalufx_engine::horizon::inverse_volatility;
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

/// Spreads up to this many ulps of a value's magnitude are rounding noise.
pub(crate) const NOISE_ULPS: f64 = 64.0;

/// Whether every value equals the first to within rounding.
fn is_constant(data: &[f64]) -> bool {
    let (lo, hi) =
        data.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &x| (lo.min(x), hi.max(x)));
    hi - lo <= NOISE_ULPS * f64::EPSILON * lo.abs().max(hi.abs())
}

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
