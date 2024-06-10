/// This module will return errors if the data fetching process fails due to
/// network issues, invalid ticker symbols, or issues with the data source API.
pub mod fetch_data_svc;

/// This module will return errors if the LLM API requests fail due to
/// network issues, authentication errors, or invalid input data.
pub mod llm_svc;

/// This module will return errors if the data processing tasks fail due to
/// invalid input data, mathematical errors, or insufficient data for analysis.
pub mod processing_svc;
