pub mod bellwether_stock_analysis_1987_example;
pub mod diversified_etf_portfolio_optimization;
pub mod fetch_data_example;
pub mod generate_market_analysis_report;
pub mod logger;
pub mod technical_analysis_indicators;

use nalufx::errors::NaluFxError;
use std::io::{stdin, BufRead};

pub(crate) fn main() -> Result<(), NaluFxError> {
    logger::main();

    // Print the options to the user
    println!("Choose an example to run:");
    println!("1. Analyze Bellwether Stock");
    println!("2. Optimise ETF Portfolio");
    println!("3. Fetch Stock Data");
    println!("4. Generate Technical Analysis");
    println!("5. Generate Market Analysis Report");
    println!("6. Exit");
    println!("Enter the number of the example you want to run: ");

    // Read the user's input
    let mut input = String::new();
    stdin()
        .lock()
        .read_line(&mut input)
        .map_err(NaluFxError::InputError)?;

    // Determine which example to run based on the user's input
    match input.trim().parse() {
        Ok(1) => bellwether_stock_analysis_1987_example::main(),
        Ok(2) => diversified_etf_portfolio_optimization::main(),
        Ok(3) => fetch_data_example::main(),
        Ok(4) => {
            if let Err(e) = technical_analysis_indicators::main() {
                eprintln!("Error during technical analysis: {}", e);
                return Err(NaluFxError::TechnicalAnalysisError(format!("{}", e)));
            }
        }
        Ok(5) => generate_market_analysis_report::main(),
        Ok(6) => {
            println!("Exiting...");
            return Ok(());
        }
        _ => return Err(NaluFxError::InvalidOption),
    }

    Ok(())
}
