use reqwest::Client;
use std::error::Error;
use time::OffsetDateTime;
use tracing::{error, info};
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
/// * `start_date` - An optional `OffsetDateTime` representing the start date for the data retrieval.
/// * `end_date` - An optional `OffsetDateTime` representing the end date for the data retrieval.
///
/// # Returns
///
/// This function returns a `Result` containing a vector of closing prices (`Vec<f64>`) if successful,
/// or an error (`Box<dyn Error>`) if the data retrieval fails.
///
/// # Examples
///
/// ```
/// use time::{Duration, OffsetDateTime};
/// use nalufx::services::fetch_data_svc::fetch_data;
///
/// #[tokio::main]
/// async fn main() {
///     let start_date = Some(OffsetDateTime::now_utc() - Duration::days(30));
///   let end_date = Some(OffsetDateTime::now_utc());
///   match fetch_data("AAPL", start_date, end_date).await {
///   Ok(data) => println!("Data: {:?}", data),
///   Err(e) => eprintln!("Error: {}", e),
///   }
/// }
/// ```
pub async fn fetch_data(
    ticker: &str,
    start_date: Option<OffsetDateTime>,
    end_date: Option<OffsetDateTime>,
) -> Result<Vec<f64>, Box<dyn Error>> {
    info!("Attempting to fetch data for ticker: {}", ticker);

    let client = Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
        .build()?;

    let start_date = start_date.map_or(0, OffsetDateTime::unix_timestamp);
    let end_date = end_date.unwrap_or_else(OffsetDateTime::now_utc).unix_timestamp();

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
                        },
                        Err(e) => {
                            error!("Failed to parse quotes for ticker {}: {}", ticker, e);
                            Err(Box::new(e))
                        },
                    },
                    Err(e) => {
                        error!("Failed to parse response JSON for ticker {}: {}", ticker, e);
                        Err(Box::new(e))
                    },
                }
            } else {
                error!("Request failed with status: {}", response.status());
                Err(Box::new(std::io::Error::other("Request failed")))
            }
        },
        Err(e) => {
            error!("Failed to send request for ticker {}: {}", ticker, e);
            Err(Box::new(e))
        },
    }
}

/// Daily returns for each ticker over its full available history, as a
/// matrix with one column per ticker that has usable data.
///
/// Tickers that fail to fetch, or have a bad price, are logged and left
/// out; the rest are aligned by
/// [`align_returns`](crate::services::processing_svc::align_returns).
///
/// # Errors
///
/// [`crate::errors::NaluFxError::NaluFxError`] if no ticker has at least
/// two returns in common with the others.
pub async fn fetch_returns(
    tickers: &[String],
) -> Result<(Vec<String>, ndarray::Array2<f64>), crate::errors::NaluFxError> {
    use crate::services::processing_svc::{align_returns, calculate_daily_returns};
    let mut names = Vec::new();
    let mut series = Vec::new();
    for ticker in tickers {
        let returns = match fetch_data(ticker, None, None).await {
            Ok(closes) => calculate_daily_returns(&closes).map_err(|e| e.to_string()),
            Err(e) => Err(e.to_string()),
        };
        match returns {
            Ok(r) => {
                names.push(ticker.clone());
                series.push(r);
            },
            Err(e) => error!("Skipping {ticker}: {e}"),
        }
    }
    let matrix = align_returns(&series)
        .map_err(|e| crate::errors::NaluFxError::NaluFxError(e.to_string()))?;
    Ok((names, matrix))
}
