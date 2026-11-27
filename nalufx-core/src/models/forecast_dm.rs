//! Forecasts and their prediction intervals.

use serde::{Deserialize, Serialize};

/// A point forecast with its prediction interval, one value per day.
///
/// `lower[i] <= point[i] <= upper[i]` for every day `i`; the interval
/// holds the true value with probability `level` under the fitted model.
///
/// # Examples
///
/// ```
/// use nalufx::models::forecast_dm::IntervalForecast;
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
