/// This module will return errors if the data fetching process fails due to
/// network issues, invalid ticker symbols, or issues with the data source API.
pub mod fetch_data;

/// This module will return errors if the data processing tasks fail due to
/// invalid input data, mathematical errors, or insufficient data for analysis.
pub mod processing;
