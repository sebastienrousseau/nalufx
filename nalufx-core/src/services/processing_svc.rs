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
/// # Examples
///
/// ```
/// use nalufx::services::processing_svc::calculate_daily_returns;
/// let closes = vec![100.0, 101.0, 102.0, 101.5];
/// let daily_returns = calculate_daily_returns(&closes);
/// assert_eq!(daily_returns, vec![0.010000000000000009, 0.00990099009900991, -0.004901960784313708]);
/// ```
pub fn calculate_daily_returns(closes: &[f64]) -> Vec<f64> {
    closes.windows(2).map(|w| (w[1] / w[0]) - 1.0).collect()
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
