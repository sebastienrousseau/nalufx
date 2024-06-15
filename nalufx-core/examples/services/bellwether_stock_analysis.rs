//! # Bellwether Stock Analysis Example
//!
//! This example demonstrates the analysis of a bellwether stock using historical data, market indices, fund characteristics, and advanced data analysis techniques.
//!
//! Usage:
//!
//! 1. Run the code using `cargo run --example bellwether_stock_analysis_1987_example`.
//! 2. Enter the ticker symbol for a bellwether stock when prompted.
//! 3. Enter the initial investment amount when prompted.
//! 4. Enter the start date (YYYY-MM-DD) for the analysis period when prompted.
//! 5. Enter the end date (YYYY-MM-DD) for the analysis period when prompted.
//! 6. The code will fetch historical data, perform analysis, and generate a report with investment recommendations.
//!
use nalufx_llms::llms::{LLM, openai, openai::OpenAI};
use nalufx::{
    errors::NaluFxError,
    utils::input::get_input,
};
use reqwest::Client;
use nalufx::services::bellwether_stock_analysis_svc::generate_analysis;

#[tokio::main]
pub(crate) async fn main() -> Result<(), NaluFxError> {
    // Get user input for LLM choice
    let llm_choice = get_input("Enter the LLM to use (e.g., openai, claude, gemini, llama, mistral, ollama):")?;
    let (llm, api_key): (Box<dyn LLM>, String) = match llm_choice.as_str() {
        "openai" => {
            let api_key = match openai::get_openai_api_key() {
                Ok(key) => key,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return Err(NaluFxError::InvalidData);
                }
            };
            (Box::new(OpenAI), api_key)
        },
        // Add other cases for different LLMs with their respective API key functions
        _ => {
            eprintln!("Unsupported LLM choice");
            return Err(NaluFxError::InvalidOption);
        }
    };

    // Get user input for ticker, initial investment amount, start date, and end date
    let ticker_input = get_input("Enter the ticker symbol for a bellwether stock:")?;
    let ticker = match nalufx::utils::ticker::validate_ticker(&ticker_input) {
        Ok(symbol) => symbol.to_string(),
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(NaluFxError::InvalidOption);
        }
    };

    // Get user input for the initial investment amount
    let initial_investment_input = get_input("Enter the initial investment amount:")?;
    let initial_investment = match nalufx::utils::validation::validate_positive_float(&initial_investment_input) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(NaluFxError::InvalidOption);
        }
    };

    let start_date_input = get_input("Enter the start date (YYYY-MM-DD):")?;
    let end_date_input = get_input("Enter the end date (YYYY-MM-DD):")?;

    // Call the generate_analysis function from the new service
    generate_analysis(
        llm,
        &Client::new(),
        &api_key,
        &ticker,
        initial_investment,
        &start_date_input,
        &end_date_input,
    )
    .await
}
