/// Bellwether Stock Analysis Example
pub mod bellwether_stock_analysis;
/// Diversified ETF Portfolio Optimization Example
pub mod diversified_etf_portfolio_optimization;
/// ESG Portfolio Optimization Example
pub mod esg_portfolio_optimization;
/// Factor Investing Stock Ranking Example
pub mod factor_investing_stock_ranking;
/// Fetch Stock Data Example
pub mod fetch_stock_data;
/// Generate Portfolio Report Example
pub mod generate_portfolio_report;
/// Logger Module
pub mod logger;
/// Mean-Variance Portfolio Optimization Example
pub mod mean_variance_optimization;
/// Risk Parity Portfolio Optimization Example
pub mod risk_parity_portfolio_optimization;
/// Technical Analysis Indicators Example
pub mod technical_analysis_indicators;

/// Imports
use nalufx::errors::NaluFxError;
use nalufx::{macro_ascii, utils::input::get_input};

/// The main function of the application.
/// It provides a menu for the user to choose an example to run.
///
/// # Errors
///
/// Returns a `NaluFxError` if any of the following occurs:
/// - An error occurs while reading user input.
/// - An error occurs during the execution of a specific example.
pub(crate) fn main() -> Result<(), NaluFxError> {
    // Initialize the logger
    logger::main();

    // Printing the ASCII art to the console
    println!("{}", macro_ascii!("NaluFX"));

    // Print the options to the user
    println!("Choose an example to run:\n");
    println!("1. Analyze Bellwether Stock - Perform in-depth analysis of a key stock to gauge market trends.");
    println!("2. Optimize ETF Portfolio - Find the optimal allocation for your ETF investments.");
    println!("3. Create ESG Portfolio - Build a socially responsible investment portfolio.");
    println!("4. Rank Stocks by Factors - Rank stocks based on key financial factors.");
    println!("5. Retrieve Stock Data - Fetch historical data for specified stocks.");
    println!("6. Generate Portfolio Report - Generate a comprehensive report for your investment portfolio.");
    println!("7. Optimize Mean-Variance Portfolio - Use the mean-variance approach for portfolio optimization.");
    println!("8. Balance Risk Parity Portfolio - Allocate risk equally across all assets in your portfolio.");
    println!("9. Perform Technical Analysis - Generate technical indicators for stocks to inform trading decisions.");
    println!("0. Quit - Exit the application.");

    // Read the user's input
    let input = match get_input("\nEnter the number of the example you want to run: ") {
        Ok(input) => input,
        Err(e) => {
            eprintln!("Error reading input: {}", e);
            return Err(e);
        }
    };

    // Determine which example to run based on the user's input
    match input.trim().parse() {
        Ok(1) => bellwether_stock_analysis::main()?,
        Ok(2) => diversified_etf_portfolio_optimization::main()?,
        Ok(3) => esg_portfolio_optimization::main()?,
        Ok(4) => factor_investing_stock_ranking::main()?,
        Ok(5) => fetch_stock_data::main()?,
        Ok(6) => generate_portfolio_report::main()?,
        Ok(7) => mean_variance_optimization::main()?,
        Ok(8) => risk_parity_portfolio_optimization::main()?,
        Ok(9) => technical_analysis_indicators::main()?,
        Ok(0) => {
            println!("Exiting NaluFX, goodbye!");
            return Ok(());
        }
        _ => return Err(NaluFxError::InvalidOption),
    };

    Ok(())
}
