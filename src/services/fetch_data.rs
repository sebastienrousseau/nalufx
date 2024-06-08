use chrono::{DateTime, Utc};
use log::{error, info};
use reqwest::Client;
use std::error::Error;
use yahoo_finance_api as yahoo;

/// Fetches historical data for a given ticker symbol from Yahoo Finance.
///
/// This asynchronous function retrieves historical closing prices for the specified ticker
/// symbol within the optional date range provided. If no date range is specified, it fetches
/// data from the earliest available date to the current date.
///
/// # Arguments
///
/// * `ticker` - A string slice that holds the ticker symbol of the stock (e.g., "AAPL").
/// * `start_date` - An optional `DateTime<Utc>` representing the start date for the data retrieval.
/// * `end_date` - An optional `DateTime<Utc>` representing the end date for the data retrieval.
///
/// # Returns
///
/// This function returns a `Result` containing a vector of closing prices (`Vec<f64>`) if successful,
/// or an error (`Box<dyn Error>`) if the data retrieval fails.
///
/// # Examples
///
/// ```
/// use chrono::Utc;
/// use nalufx::services::fetch_data::fetch_data;
///
/// #[tokio::main]
/// async fn main() {
///     let start_date = Some(Utc::now() - chrono::Duration::days(30));
///     let end_date = Some(Utc::now());
///     match fetch_data("AAPL", start_date, end_date).await {
///         Ok(data) => println!("Data: {:?}", data),
///         Err(e) => eprintln!("Error: {}", e),
///     }
/// }
/// ```
pub async fn fetch_data(
    ticker: &str,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
) -> Result<Vec<f64>, Box<dyn Error>> {
    info!("Attempting to fetch data for ticker: {}", ticker);

    let client = Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
        .build()?;

    let start_date = start_date.map_or(0, |date| date.timestamp());
    let end_date = end_date.map_or(Utc::now().timestamp(), |date| date.timestamp());

    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?period1={}&period2={}&interval=1d",
        ticker, start_date, end_date
    );

    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<yahoo::YResponse>().await {
                    Ok(yresponse) => match yresponse.quotes() {
                        Ok(quotes) => {
                            let closes: Vec<f64> = quotes.iter().map(|quote| quote.close).collect();
                            info!("Successfully parsed closing prices: {:?}", closes);
                            Ok(closes)
                        }
                        Err(e) => {
                            error!("Failed to parse quotes for ticker {}: {}", ticker, e);
                            Err(Box::new(e))
                        }
                    },
                    Err(e) => {
                        error!("Failed to parse response JSON for ticker {}: {}", ticker, e);
                        Err(Box::new(e))
                    }
                }
            } else {
                error!(
                    "Request failed with status: {}",
                    response.status().to_string()
                );
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Request failed",
                )))
            }
        }
        Err(e) => {
            error!("Failed to send request for ticker {}: {}", ticker, e);
            Err(Box::new(e))
        }
    }
}
