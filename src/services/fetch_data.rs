use log::{error, info};
use reqwest::Client;
use std::error::Error;
use yahoo_finance_api as yahoo;

pub async fn fetch_data(ticker: &str) -> Result<Vec<f64>, Box<dyn Error>> {
    info!("Attempting to fetch data for ticker: {}", ticker);

    let client = Client::builder()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
        .build()?;

    match client
        .get(&format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?range=1y&interval=1d",
            ticker
        ))
        .send()
        .await
    {
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
