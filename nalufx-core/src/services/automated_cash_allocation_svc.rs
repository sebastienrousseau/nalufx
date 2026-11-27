use crate::errors::AllocationError;
use crate::models::allocation_dm::AllocationOrder;
use crate::utils::currency::format_currency;
use nalufx_llms::llms::LLM;
use reqwest::Client;
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

/// This function generates a comprehensive analysis report for a given portfolio.
///
/// # Arguments
///
/// * `llm` - A boxed trait object implementing the LLM trait.
/// * `client` - A reference to a reqwest::Client instance.
/// * `api_key` - A reference to a string representing the API key for the LLM service.
/// * `request` - The portfolio and investor inputs, see [`AnalysisRequest`].
///
/// # Returns
///
/// * `Result<String, Box<dyn std::error::Error>>` - A Result containing the generated report as a string on success, or an error on failure.
///
/// # Errors
///
/// [`AllocationError::InvalidPrice`] if a start price is zero, negative or
/// not finite, so its return cannot be computed; or any error from the
/// model provider.
pub async fn generate_analysis(
    llm: Box<dyn LLM>,
    client: &Client,
    api_key: &str,
    request: AnalysisRequest<'_>,
) -> Result<String, Box<dyn std::error::Error>> {
    let AnalysisRequest {
        portfolio_name,
        etf_allocation,
        mutual_fund_allocation,
        values_input,
        financial_objectives_input,
        start_date,
        end_date,
        real_time_prices,
    } = request;

    let allocations_str = etf_allocation
        .iter()
        .map(|order| {
            format!("{}: {} ({})", order.name, format_currency(order.amount), order.symbol)
        })
        .chain(mutual_fund_allocation.iter().map(|order| {
            format!("{}: {} ({})", order.name, format_currency(order.amount), order.symbol)
        }))
        .collect::<Vec<_>>()
        .join("\n");

    let performance_str = real_time_prices
        .iter()
        .map(|(symbol, &(start_price, end_price))| {
            Ok(format!(
                "{}: Start Price: {}, End Price: {}, Return: {:.2}%",
                symbol,
                format_currency(start_price),
                format_currency(end_price),
                percent_return(start_price, end_price)?
            ))
        })
        .collect::<Result<Vec<_>, AllocationError>>()?
        .join("\n");

    let prompt = format!(
        "Portfolio Name: {}\n\nPortfolio Allocations:\n{}\n\nInvestor Values: {}\nFinancial Objectives: {}\nStart Date: {}\nEnd Date: {}\n\nPerformance:\n{}",
        portfolio_name, allocations_str, values_input, financial_objectives_input, start_date, end_date, performance_str
    );

    let response = llm.send_request(client, api_key, &prompt, 1500).await?;

    let generated_report =
        response["choices"][0]["message"]["content"].as_str().unwrap_or("").to_string();

    Ok(generated_report)
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
    fn a_zero_start_price_is_an_error_not_infinity() {
        assert_eq!(percent_return(0.0, 110.0), Err(AllocationError::InvalidPrice(0.0)));
        assert!(percent_return(f64::NAN, 1.0).is_err());
        assert!(percent_return(-1.0, 1.0).is_err());
    }
}
