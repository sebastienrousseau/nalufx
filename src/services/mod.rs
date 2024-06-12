/// This module will return errors if the data fetching process fails due to
/// network issues, invalid ticker symbols, or issues with the data source API.
pub mod fetch_data_svc;

/// This module will return errors if the automated cash allocation process fails due to insufficient data for analysis, mathematical errors, or invalid input data.
pub mod automated_cash_allocation_svc;

/// This module will return errors if the bellwether stock analysis process fails due to insufficient data for analysis, mathematical errors, or invalid input data.
pub mod bellwether_stock_analysis_svc;

/// This module will return errors if the data processing tasks fail due to
/// invalid input data, mathematical errors, or insufficient data for analysis.
pub mod processing_svc;
