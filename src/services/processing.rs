pub fn calculate_daily_returns(closes: &[f64]) -> Vec<f64> {
    closes.windows(2).map(|w| (w[1] / w[0]) - 1.0).collect()
}

pub fn calculate_cash_flows(daily_returns: &[f64], initial_investment: f64) -> Vec<f64> {
    daily_returns
        .iter()
        .map(|&r| r * initial_investment)
        .collect()
}
