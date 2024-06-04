/// Example for the `fetch_data` service
// pub mod fetch_data_example;

/// Example of a Diversified ETF Portfolio Optimization Using Advanced Data Analysis and Reinforcement Learning
// pub mod diversified_etf_portfolio_optimization;

/// Example for the `bellwether_stock_analysis_1987` service
// pub mod bellwether_stock_analysis_1987_example;

/// Example for the `technical_analysis_indicators` service
pub mod technical_analysis_indicators;

/// Example for the `logger` service
pub mod logger;

pub(crate) fn main() {
    logger::main();

    // bellwether_stock_analysis_1987_example::main();

    // diversified_etf_portfolio_optimization::main();

    // fetch_data_example::main();

    if let Err(e) = technical_analysis_indicators::main() {
        eprintln!("Error: {}", e);
    }
}
