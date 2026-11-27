use crate::errors::AllocationError;
use ndarray::Array2;

/// Calculates the daily returns from a slice of closing prices.
///
/// This function takes a slice of closing prices and calculates the daily returns
/// as the percentage change from one day to the next. The return for each day is
/// calculated as `(current_day_close / previous_day_close) - 1.0`.
///
/// # Arguments
///
/// * `closes` - A slice of closing prices (`&[f64]`). Each entry represents the closing price of an asset at the end of a trading day.
///
/// # Returns
///
/// A vector of daily returns (`Vec<f64>`) where each entry represents the return for a given day.
///
/// # Errors
///
/// Returns [`AllocationError::InvalidPrice`] for the first close that is
/// zero, negative, NaN or infinite: a return from it would be infinite or
/// meaningless.
///
/// # Examples
///
/// ```
/// use nalufx::services::processing_svc::calculate_daily_returns;
/// let closes = vec![100.0, 101.0, 102.0, 101.5];
/// let daily_returns = calculate_daily_returns(&closes).unwrap();
/// assert_eq!(daily_returns, vec![0.010000000000000009, 0.00990099009900991, -0.004901960784313708]);
/// assert!(calculate_daily_returns(&[0.0, 1.0]).is_err());
/// ```
pub fn calculate_daily_returns(closes: &[f64]) -> Result<Vec<f64>, AllocationError> {
    if let Some(&bad) = closes.iter().find(|c| !(c.is_finite() && **c > 0.0)) {
        return Err(AllocationError::InvalidPrice(bad));
    }
    Ok(closes.windows(2).map(|w| (w[1] / w[0]) - 1.0).collect())
}

/// Calculates the cash flows from daily returns and an initial investment.
///
/// This function takes a slice of daily returns and an initial investment amount,
/// and calculates the cash flows for each day based on the daily returns.
/// The cash flow for each day is calculated as `daily_return * initial_investment`.
///
/// # Arguments
///
/// * `daily_returns` - A slice of daily returns (`&[f64]`). Each entry represents the return for a given day.
/// * `initial_investment` - A floating-point value representing the initial investment amount (`f64`).
///
/// # Returns
///
/// A vector of cash flows (`Vec<f64>`) where each entry represents the cash flow for a given day.
///
/// # Examples
///
/// ```
/// use nalufx::services::processing_svc::calculate_cash_flows;
/// let daily_returns = vec![0.01, 0.009900990099009901, -0.004901960784313725];
/// let initial_investment = 1000.0;
/// let cash_flows = calculate_cash_flows(&daily_returns, initial_investment);
/// assert_eq!(cash_flows, vec![10.0, 9.900990099009901, -4.901960784313726]);
/// ```
pub fn calculate_cash_flows(daily_returns: &[f64], initial_investment: f64) -> Vec<f64> {
    daily_returns.iter().map(|&r| r * initial_investment).collect()
}

/// Daily-return series as a matrix, one column per series, aligned on
/// their most recent common length.
///
/// Price feeds that return closes without dates can only be aligned by
/// position: this keeps the last `n` returns of each series, where `n` is
/// the shortest, and assumes the series share a trading calendar.
///
/// # Errors
///
/// - [`AllocationError::EmptyInput`] if there are no series.
/// - [`AllocationError::InvalidParameter`] if the shortest has fewer than
///   two returns.
///
/// # Examples
///
/// ```
/// use nalufx::services::processing_svc::align_returns;
///
/// let m = align_returns(&[vec![0.1, 0.2, 0.3], vec![0.5, 0.6]]).unwrap();
/// assert_eq!(m.column(0).to_vec(), vec![0.2, 0.3]);
/// assert_eq!(m.column(1).to_vec(), vec![0.5, 0.6]);
/// ```
pub fn align_returns(series: &[Vec<f64>]) -> Result<Array2<f64>, AllocationError> {
    let n = series.iter().map(Vec::len).min().ok_or(AllocationError::EmptyInput)?;
    if n < 2 {
        return Err(AllocationError::InvalidParameter(format!(
            "the shortest series has {n} returns; at least 2 are needed"
        )));
    }
    Ok(Array2::from_shape_fn((n, series.len()), |(i, j)| series[j][series[j].len() - n + i]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cash_flows_scale_the_returns_by_the_investment() {
        assert_eq!(calculate_cash_flows(&[0.01, -0.02], 1000.0), vec![10.0, -20.0]);
    }

    #[test]
    fn a_bad_close_is_named_in_the_error() {
        for bad in [0.0, -5.0, f64::INFINITY] {
            assert_eq!(
                calculate_daily_returns(&[100.0, bad]),
                Err(AllocationError::InvalidPrice(bad))
            );
        }
    }

    #[test]
    fn align_returns_rejects_no_series_and_a_short_one() {
        assert_eq!(align_returns(&[]), Err(AllocationError::EmptyInput));
        assert!(matches!(
            align_returns(&[vec![0.1, 0.2], vec![0.3]]),
            Err(AllocationError::InvalidParameter(_))
        ));
    }
}
