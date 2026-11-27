//! Time series forecasts with prediction intervals.
//!
//! [`forecast_time_series`] fits a non-seasonal AutoETS model;
//! [`forecast_seasonal`] decomposes weekly, monthly or other cycles with
//! MSTL first. Both are deterministic and always return an interval.

use crate::{check_empty_inputs, check_invalid_data, AllocationError};
use augurs_core::{Fit, Predict};
use augurs_ets::{trend::AutoETSTrendModel, AutoETS};
use augurs_mstl::MSTLModel;
use serde::{Deserialize, Serialize};

/// The probability every NaluFX prediction interval covers.
pub const FORECAST_LEVEL: f64 = 0.95;

/// A point forecast with its prediction interval, one value per day.
///
/// `lower[i] <= point[i] <= upper[i]` for every day `i`; the interval
/// holds the true value with probability `level` under the fitted model.
///
/// # Examples
///
/// ```
/// use nalufx_engine::forecast::IntervalForecast;
///
/// let forecast = IntervalForecast {
///     level: 0.95,
///     point: vec![1.0, 1.1],
///     lower: vec![0.8, 0.85],
///     upper: vec![1.2, 1.35],
/// };
/// assert_eq!(forecast.widths(), vec![1.2 - 0.8, 1.35 - 0.85]);
/// ```
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct IntervalForecast {
    /// The probability the interval covers, such as `0.95`.
    pub level: f64,
    /// The forecast value for each day.
    pub point: Vec<f64>,
    /// The lower bound of the interval for each day.
    pub lower: Vec<f64>,
    /// The upper bound of the interval for each day.
    pub upper: Vec<f64>,
}

impl IntervalForecast {
    /// The width of the interval, `upper - lower`, for each day.
    #[must_use]
    pub fn widths(&self) -> Vec<f64> {
        self.upper.iter().zip(&self.lower).map(|(u, l)| u - l).collect()
    }
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
/// use nalufx_engine::forecast::forecast_time_series;
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

/// Forecasts a series with weekly, monthly or other cycles, with its 95%
/// prediction interval.
///
/// MSTL splits the series into one seasonal component per period and a
/// trend, forecasts the trend with AutoETS, and adds the seasonal
/// components back. Use it for cash flows that repeat on a schedule
/// (payroll every 5 business days: `periods = [5]`; month-end: `[21]`).
///
/// # Arguments
///
/// * `data` - Historical values, oldest first; at least two full cycles of
///   the longest period.
/// * `periods` - The length of each cycle, in observations; each at least 2.
/// * `num_days` - The number of days to forecast.
///
/// # Errors
///
/// - As [`forecast_time_series`].
/// - [`AllocationError::InvalidParameter`] if `periods` is empty, a period is
///   under 2, or the data is shorter than two cycles of the longest period.
///
/// # Examples
///
/// ```
/// use nalufx_engine::forecast::forecast_seasonal;
///
/// // Four weeks of a 5-day cycle on a gentle trend.
/// let data: Vec<f64> = (0..40).map(|i| 100.0 + i as f64 * 0.5 + [0.0, 3.0, 1.0, -2.0, 6.0][i % 5]).collect();
/// let forecast = forecast_seasonal(&data, &[5], 5).unwrap();
/// assert_eq!(forecast.point.len(), 5);
/// // The cycle carries on: day 5 of the cycle is the high one.
/// assert!(forecast.point[4] > forecast.point[3]);
/// ```
pub fn forecast_seasonal(
    data: &[f64],
    periods: &[usize],
    num_days: usize,
) -> Result<IntervalForecast, AllocationError> {
    validate_series(data, num_days)?;
    let longest = periods.iter().copied().max().unwrap_or(0);
    if periods.is_empty() || periods.iter().any(|&p| p < 2) {
        return Err(AllocationError::InvalidParameter(
            "seasonal periods must be non-empty and each at least 2".to_string(),
        ));
    }
    if data.len() < 2 * longest {
        return Err(AllocationError::InvalidParameter(format!(
            "{} observations are fewer than two cycles of the {longest}-observation period",
            data.len()
        )));
    }
    let trend: AutoETSTrendModel = AutoETS::new(1, "ZZN").map_err(failed)?.into();
    let model = MSTLModel::new(periods.to_vec(), trend).fit(data).map_err(failed)?;
    to_interval_forecast(model.predict(num_days, FORECAST_LEVEL).map_err(failed)?)
}

/// Wraps a forecasting failure.
fn failed(err: impl std::fmt::Display) -> AllocationError {
    AllocationError::ForecastingError(err.to_string())
}

/// Checks a series and a horizon before anything is computed from them.
pub(crate) fn validate_series(data: &[f64], num_days: usize) -> Result<(), AllocationError> {
    check_empty_inputs!(data)?;
    check_invalid_data!(data)?;
    if num_days == 0 {
        return Err(AllocationError::InvalidHorizon);
    }
    Ok(())
}
