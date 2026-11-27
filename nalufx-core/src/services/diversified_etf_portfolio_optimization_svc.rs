//! Diversified ETF report: picks the ETF with the best risk-adjusted
//! historical return and allocates an investment across the next few
//! days by inverse forecast volatility.

use crate::{
    errors::NaluFxError,
    services::{fetch_data_svc::fetch_data, processing_svc::calculate_daily_returns},
    utils::{
        calculations::{calculate_optimal_allocation, DEFAULT_HORIZON_DAYS},
        currency::format_currency,
        date::format_date,
    },
};
use std::fs::File;
use std::io::Write;
use textplots::{Chart, LabelBuilder, LabelFormat, Plot, Shape};
use time::{Duration, OffsetDateTime};

/// Generates a report for a set of ETFs and writes it to `./reports/`.
///
/// Each ETF's daily returns come from its full available price history.
/// The ETF with the highest ratio of mean to standard deviation of daily
/// returns (an unannualised Sharpe ratio with a zero risk-free rate) is
/// selected, and the investment is spread across the next
/// [`DEFAULT_HORIZON_DAYS`] days by inverse forecast volatility.
///
/// # Arguments
///
/// * `tickers` - The ETF tickers to compare.
/// * `initial_investment` - The amount the allocation is applied to.
///
/// # Errors
///
/// - [`NaluFxError::InputError`] if the report file cannot be written.
/// - [`NaluFxError::PortfolioOptimizationError`] if the selected ETF's allocation fails.
pub async fn generate_analysis(
    tickers: Vec<String>,
    initial_investment: f64,
) -> Result<(), NaluFxError> {
    let today = OffsetDateTime::now_utc();
    let filename =
        format!("./reports/{}_03_diversified_etf_portfolio_optimization.md", format_date(today));
    let mut file = File::create(&filename)?;

    let mut returns = Vec::new();
    for ticker in &tickers {
        match fetch_data(ticker, None, None).await {
            Ok(closes) => match calculate_daily_returns(&closes) {
                Ok(daily) => returns.push((ticker.clone(), daily)),
                Err(e) => eprintln!("Skipping {ticker}: {e}"),
            },
            Err(e) => eprintln!("Error fetching data for ticker {ticker}: {e}"),
        }
    }

    let Some((best, ratio)) = select_best(&returns) else {
        return emit(&mut file, "No ETF has enough price history to compare.");
    };
    let allocation = calculate_optimal_allocation(&returns[best].1, DEFAULT_HORIZON_DAYS)
        .map_err(|e| NaluFxError::PortfolioOptimizationError(e.to_string()))?;
    write_report(&mut file, &returns[best].0, ratio, &allocation, initial_investment, today)
}

/// The index of the series with the highest Sharpe ratio, and the ratio.
/// The first wins a tie; a series without a ratio is never selected.
fn select_best(returns: &[(String, Vec<f64>)]) -> Option<(usize, f64)> {
    returns
        .iter()
        .enumerate()
        .filter_map(|(i, (_, daily))| sharpe_ratio(daily).map(|r| (i, r)))
        .fold(None, |best, (i, r)| match best {
            Some((_, b)) if b >= r => best,
            _ => Some((i, r)),
        })
}

/// Mean over sample standard deviation of daily returns; `None` with
/// fewer than two returns or no variation.
fn sharpe_ratio(daily: &[f64]) -> Option<f64> {
    if daily.len() < 2 {
        return None;
    }
    let n = daily.len() as f64;
    let mean = daily.iter().sum::<f64>() / n;
    let variance = daily.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let ratio = mean / variance.sqrt();
    (variance > 0.0 && ratio.is_finite()).then_some(ratio)
}

/// Prints a section and writes it to the report file.
fn emit(file: &mut File, text: &str) -> Result<(), NaluFxError> {
    println!("{text}");
    writeln!(file, "{text}")?;
    Ok(())
}

/// Writes every section of the report.
fn write_report(
    file: &mut File,
    etf: &str,
    ratio: f64,
    allocation: &[f64],
    investment: f64,
    today: OffsetDateTime,
) -> Result<(), NaluFxError> {
    emit(file, "# Diversified ETF Allocation Report\n\n## Introduction\nExchange-Traded Funds (ETFs) are investment funds that trade like stocks. They hold assets such as stocks, commodities or bonds, and an arbitrage mechanism keeps their price close to their net asset value.")?;
    emit(file, &format!("\n## ETF Selection\n**{etf}** was selected: of the ETFs compared, its daily returns have the highest ratio of mean to standard deviation, {ratio:.4} (an unannualised Sharpe ratio with a zero risk-free rate). No other criterion was used."))?;
    emit(file, &format!("\n## Allocation Method\nThe daily returns of {etf} are forecast {DEFAULT_HORIZON_DAYS} days ahead with an exponential smoothing model (AutoETS). Each day is weighted by the inverse of its 95% forecast interval's width, so days the model is surer about get more. The weights sum to 100%; they reflect the forecast's uncertainty, not its expected return.\n\n- Allocation: {allocation:?}"))?;
    plot(allocation);
    emit(
        file,
        &format!(
            "\n## Allocation Recommendation\nApplied to an investment of {}:\n",
            format_currency(investment)
        ),
    )?;
    for (i, weight) in allocation.iter().enumerate() {
        let date = format_date(today + Duration::days(i64::try_from(i).unwrap_or(i64::MAX)));
        let amount = format_currency(weight * investment);
        emit(
            file,
            &format!(
                "- Day {}: {date} - Allocate {amount} ({:.2}%) to {etf}",
                i + 1,
                weight * 100.0
            ),
        )?;
    }
    emit(file, "\n## Risks and Limitations\n- **Market risk**: investments can lose value, and past performance is not indicative of future results.\n- **Concentration risk**: the whole amount goes to one ETF.\n- **Model risk**: the forecast is fitted to past prices only and does not anticipate news or regime changes.")?;
    emit(file, "\n## Disclaimer\nThis report is for research and information only and is not financial advice. Consult a financial professional before making any investment decision.\n")
}

/// Draws the allocation as a terminal line chart.
fn plot(allocation: &[f64]) {
    let points: Vec<(f32, f32)> =
        allocation.iter().enumerate().map(|(i, &w)| (i as f32, w as f32)).collect();
    let last_x = points.last().map_or(0.0, |&(x, _)| x);
    Chart::new_with_y_range(120, 60, 0.0, last_x, 0.0, 1.0)
        .lineplot(&Shape::Lines(&points))
        .x_label_format(LabelFormat::Custom(Box::new(|x| format!("Day {}", x as usize + 1))))
        .y_label_format(LabelFormat::Custom(Box::new(|y| format!("{y:.2}"))))
        .display();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sharpe_ratio_is_mean_over_sample_deviation() {
        let ratio = sharpe_ratio(&[0.01, 0.03]).expect("two varying returns have a ratio");
        // mean 0.02, sample deviation sqrt(0.0002) = 0.014142...
        assert!((ratio - 0.02 / 0.0002_f64.sqrt()).abs() < 1e-12);
    }

    #[test]
    fn sharpe_ratio_needs_two_varying_returns() {
        assert_eq!(sharpe_ratio(&[0.01]), None);
        assert_eq!(sharpe_ratio(&[0.01, 0.01, 0.01]), None);
    }

    #[test]
    fn the_highest_ratio_is_selected_and_the_first_wins_a_tie() {
        let returns = vec![
            ("A".to_string(), vec![0.01, 0.03]),
            ("B".to_string(), vec![0.02, 0.04]),
            ("C".to_string(), vec![0.02, 0.04]),
            ("D".to_string(), vec![0.05]),
        ];
        assert_eq!(select_best(&returns).map(|(i, _)| i), Some(1));
        assert_eq!(select_best(&returns[3..]), None);
    }
}
