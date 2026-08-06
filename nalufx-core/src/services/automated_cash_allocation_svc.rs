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
        .map(|(symbol, (start_price, end_price))| {
            format!(
                "{}: Start Price: {}, End Price: {}, Return: {:.2}%",
                symbol,
                format_currency(*start_price),
                format_currency(*end_price),
                ((*end_price - *start_price) / *start_price) * 100.0
            )
        })
        .collect::<Vec<_>>()
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
