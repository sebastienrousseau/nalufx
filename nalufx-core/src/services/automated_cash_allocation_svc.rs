use crate::errors::{AllocationError, NaluFxError};
use crate::models::allocation_dm::AllocationOrder;
use nalufx_llms::provider::LlmProvider;
use nalufx_llms::report::{self, Report, ReportInput};
use serde_json::json;
use std::collections::HashMap;

/// The portfolio and investor inputs a report is generated from.
///
/// Grouped into a struct rather than passed as eleven positional
/// parameters: the previous signature tripped `clippy::too_many_arguments`,
/// and six of its parameters were `&str`, so transposing any two of them
/// would still have compiled.
#[derive(Debug, Clone, Copy)]
pub struct AnalysisRequest<'a> {
    /// Name of the portfolio being analysed.
    pub portfolio_name: &'a str,
    /// The ETF allocations.
    pub etf_allocation: &'a [AllocationOrder],
    /// The mutual fund allocations.
    pub mutual_fund_allocation: &'a [AllocationOrder],
    /// The investor's stated values.
    pub values_input: &'a str,
    /// The investor's stated financial objectives.
    pub financial_objectives_input: &'a str,
    /// Start date of the analysis period.
    pub start_date: &'a str,
    /// End date of the analysis period.
    pub end_date: &'a str,
    /// Real-time prices of the assets, keyed by ticker.
    pub real_time_prices: &'a HashMap<String, (f64, f64)>,
}

/// Asks a language model for a narrative report on a portfolio.
///
/// The figures (allocations, each asset's start and end price and return,
/// the dates) are computed here and handed to the model as JSON; the
/// model writes the narrative only, and its reply must match the
/// [`AllocationReport`](nalufx_llms::report::AllocationReport) schema.
///
/// # Arguments
///
/// * `provider` - The language model that writes the narrative.
/// * `request` - The portfolio and investor inputs, see [`AnalysisRequest`].
///
/// # Errors
///
/// - [`NaluFxError::Allocation`] wrapping [`AllocationError::InvalidPrice`]
///   if a start price is zero, negative or not finite.
/// - [`NaluFxError::Llm`] if the provider fails or its reply does not
///   match the schema.
pub async fn generate_analysis(
    provider: &dyn LlmProvider,
    request: AnalysisRequest<'_>,
) -> Result<Report, NaluFxError> {
    let input = ReportInput {
        portfolio: request.portfolio_name.to_string(),
        facts: facts(&request)?,
        context: format!(
            "Values: {}. Objectives: {}.",
            request.values_input, request.financial_objectives_input
        ),
    };
    Ok(report::generate(provider, &input).await?)
}

/// The computed figures the model is given, in a stable order.
fn facts(request: &AnalysisRequest<'_>) -> Result<serde_json::Value, AllocationError> {
    let orders = |orders: &[AllocationOrder]| -> Vec<serde_json::Value> {
        orders
            .iter()
            .map(|o| json!({ "symbol": o.symbol, "name": o.name, "amount": o.amount }))
            .collect()
    };
    let mut performance = request
        .real_time_prices
        .iter()
        .map(|(symbol, &(start, end))| {
            Ok(json!({ "symbol": symbol, "start_price": start, "end_price": end, "return_percent": percent_return(start, end)? }))
        })
        .collect::<Result<Vec<_>, AllocationError>>()?;
    performance.sort_by(|a, b| a["symbol"].as_str().cmp(&b["symbol"].as_str()));
    Ok(json!({
        "period": { "start": request.start_date, "end": request.end_date },
        "etf_allocation": orders(request.etf_allocation),
        "mutual_fund_allocation": orders(request.mutual_fund_allocation),
        "performance": performance,
    }))
}

/// The return from `start` to `end`, in percent.
///
/// # Errors
///
/// [`AllocationError::InvalidPrice`] if `start` is zero, negative or not
/// finite: the division would be infinite or meaningless.
fn percent_return(start: f64, end: f64) -> Result<f64, AllocationError> {
    if !(start.is_finite() && start > 0.0) {
        return Err(AllocationError::InvalidPrice(start));
    }
    Ok((end - start) / start * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_return_is_the_change_over_the_start_price() {
        assert_eq!(percent_return(100.0, 110.0), Ok(10.0));
    }

    #[test]
    fn the_facts_are_computed_and_sorted_by_symbol() {
        let prices: HashMap<String, (f64, f64)> =
            [("VTI".to_string(), (200.0, 210.0)), ("AGG".to_string(), (100.0, 99.0))]
                .into_iter()
                .collect();
        let etfs =
            [AllocationOrder { symbol: "VTI".into(), name: "Total Market".into(), amount: 600.0 }];
        let request = AnalysisRequest {
            portfolio_name: "P",
            etf_allocation: &etfs,
            mutual_fund_allocation: &[],
            values_input: "v",
            financial_objectives_input: "o",
            start_date: "2026-01-02",
            end_date: "2026-06-30",
            real_time_prices: &prices,
        };
        let f = facts(&request).unwrap();
        assert_eq!(f["performance"][0]["symbol"], "AGG");
        assert_eq!(f["performance"][0]["return_percent"], -1.0);
        assert_eq!(f["performance"][1]["return_percent"], 5.0);
        assert_eq!(f["etf_allocation"][0]["amount"], 600.0);
        assert_eq!(f["period"]["end"], "2026-06-30");
    }

    #[test]
    fn a_zero_start_price_is_an_error_not_infinity() {
        assert_eq!(percent_return(0.0, 110.0), Err(AllocationError::InvalidPrice(0.0)));
        assert!(percent_return(f64::NAN, 1.0).is_err());
        assert!(percent_return(-1.0, 1.0).is_err());
    }
}
