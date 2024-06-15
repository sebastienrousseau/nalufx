use crate::llms::LLM;
use reqwest::Client;
use std::collections::HashMap;
use crate::utils::currency::format_currency;
use crate::models::allocation_dm::AllocationOrder;

/// This function generates a comprehensive analysis report for a given portfolio.
///
/// # Arguments
///
/// * `llm` - A boxed trait object implementing the LLM trait.
/// * `client` - A reference to a reqwest::Client instance.
/// * `api_key` - A reference to a string representing the API key for the LLM service.
/// * `portfolio_name` - A reference to a string representing the name of the portfolio.
/// * `etf_allocation` - A slice of AllocationOrder representing the ETF allocations.
/// * `mutual_fund_allocation` - A slice of AllocationOrder representing the mutual fund allocations.
/// * `values_input` - A reference to a string representing the investor's values.
/// * `financial_objectives_input` - A reference to a string representing the investor's financial objectives.
/// * `start_date` - A reference to a string representing the start date of the analysis period.
/// * `end_date` - A reference to a string representing the end date of the analysis period.
/// * `real_time_prices` - A reference to a HashMap containing the real-time prices of assets.
///
/// # Returns
///
/// * `Result<String, Box<dyn std::error::Error>>` - A Result containing the generated report as a string on success, or an error on failure.
pub async fn generate_analysis(
    llm: Box<dyn LLM>,
    client: &Client,
    api_key: &str,
    portfolio_name: &str,
    etf_allocation: &[AllocationOrder],
    mutual_fund_allocation: &[AllocationOrder],
    values_input: &str,
    financial_objectives_input: &str,
    start_date: &str,
    end_date: &str,
    real_time_prices: &HashMap<String, (f64, f64)>,
) -> Result<String, Box<dyn std::error::Error>> {
    let allocations_str = etf_allocation
        .iter()
        .map(|order| {
            format!(
                "{}: {} ({})",
                order.name,
                format_currency(order.amount),
                order.symbol
            )
        })
        .chain(mutual_fund_allocation.iter().map(|order| {
            format!(
                "{}: {} ({})",
                order.name,
                format_currency(order.amount),
                order.symbol
            )
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

    let generated_report = response["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(generated_report)
}
