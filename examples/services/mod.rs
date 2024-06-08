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
use nalufx::utils::input::get_input;

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

    // Print the options to the user
    println!("Choose an example to run:");
    println!("1. Analyze Bellwether Stock");
    println!("2. Optimise ETF Portfolio");
    println!("3. Factor Investing Stock Ranking");
    println!("4. Fetch Stock Data");
    println!("5. Generate Portfolio Report");
    println!("6. Generate Technical Analysis");
    println!("7. ESG Portfolio Optimization");
    println!("8. Risk Parity Portfolio Optimization");
    println!("9. Mean-Variance Portfolio Optimization");
    println!("0. Exit");
    println!("Enter the number of the example you want to run: ");

    // Read the user's input
    let input = match get_input("Enter the number of the example you want to run: ") {
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
            println!("Exiting...");
            return Ok(());
        }
        _ => return Err(NaluFxError::InvalidOption),
    };

    Ok(())
}
