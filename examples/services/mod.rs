/// Example for the `fetch_data` service
pub mod fetch_data_example;

/// Example for the `bellwether_stock_analysis_1987` service
// pub mod bellwether_stock_analysis_1987_example;

/// Example for the `logger` service
pub mod logger;

pub(crate) fn main() {
    logger::main();

    // bellwether_stock_analysis_1987_example::main();

    fetch_data_example::main();
}
