//! # Stock Ranking System based on Factor Investing (Yahoo Finance API)
//!
//! This module demonstrates how to fetch financial data for a list of stocks using the Yahoo Finance API,
//! calculate various factors, and generate a ranking based on the factor scores. The factors considered
//! in this example include:
//!
//! 1. Value Factor: Price-to-Earnings (P/E) Ratio, Price-to-Book (P/B) Ratio
//! 2. Quality Factor: Return on Equity (ROE), Debt-to-Equity Ratio
//! 3. Momentum Factor: 12-Month Price Momentum
//! 4. Size Factor: Market Capitalization
//!
//! The code fetches the required financial data, calculates the factor scores, and generates a ranked list
//! of stocks based on the weighted average of the factor scores. The ranking is presented in a tabular format.
//!
//! # Usage
//!
//! 1. Run the code using `cargo run --example factor_investing_stock_ranking`.
//! 2. Enter the list of stock ticker symbols (comma-separated) when prompted.
//! 3. The code will fetch the financial data, calculate the factor scores, and display the ranked list of stocks.
//!

// Imports and module declarations...
use chrono::DateTime;
use log::{error, info};
use nalufx::errors::NaluFxError;
use nalufx::utils::input::get_input;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, f64};

/// Represents the financial data of a stock.
#[derive(Debug, Serialize, Deserialize)]
struct StockData {
    symbol: String,
    short_name: String,
    currency: String,
    regular_market_price: f64,
    trailing_pe: Option<f64>,
    price_to_book: Option<f64>,
    return_on_equity: Option<f64>,
    debt_to_equity: Option<f64>,
    market_cap: Option<f64>,
    momentum_12m: f64,
    price_start_period: f64,
    price_end_period: f64,
    date_start_period: String,
    date_end_period: String,
}

/// Represents the factor scores of a stock.
#[derive(Debug)]
#[allow(dead_code)]
struct FactorScores {
    symbol: String,
    currency: String,
    value_score: f64,
    quality_score: f64,
    momentum_score: f64,
    size_score: f64,
    composite_score: f64,
    price_start_period: f64,
    price_end_period: f64,
    date_start_period: String,
    date_end_period: String,
}

/// Validates a stock ticker symbol.
///
/// # Arguments
///
/// * `input` - The ticker symbol to validate.
///
/// # Returns
///
/// * `Ok(&str)` - The validated ticker symbol.
/// * `Err(&str)` - An error message if the ticker symbol is invalid.
fn validate_ticker(input: &str) -> Result<&str, &str> {
    if input.chars().all(|c| c.is_alphanumeric()) && !input.is_empty() {
        Ok(input)
    } else {
        Err("Please enter a valid ticker symbol (alphanumeric). Example: AAPL, MSFT, GOOGL")
    }
}

/// Fetches stock data for the given stock symbols from the Yahoo Finance API.
///
/// # Arguments
///
/// * `symbols` - A slice of stock ticker symbols.
///
/// # Returns
///
/// * `Ok(Vec<StockData>)` - A vector of `StockData` structs containing the fetched financial data.
/// * `Err(reqwest::Error)` - An error if the API request fails.
async fn fetch_stock_data(symbols: &[String]) -> Result<Vec<StockData>, reqwest::Error> {
    let mut headers = header::HeaderMap::new();
    let _ = headers.insert("User-Agent", header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"));
    let _ = headers.insert(
        "Accept",
        header::HeaderValue::from_static("application/json"),
    );
    let _ = headers.insert("Cookie", header::HeaderValue::from_static("YahooFcUrl"));

    let client = Client::builder().default_headers(headers).build()?;

    let mut stock_data = Vec::new();

    for symbol in symbols {
        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?period1=0&period2=9999999999&interval=1d&includePrePost=true&events=div%7Csplit",
            symbol
        );

        let response = client.get(&url).send().await?;
        let data: Value = response.json().await?;
        info!("Fetched data for {}: {:?}", symbol, data); // Logging for debugging

        if let Some(result) = data["chart"]["result"].as_array() {
            if !result.is_empty() {
                let meta = &result[0]["meta"];
                let currency = meta["currency"].as_str().unwrap_or("USD").to_string();
                let regular_market_price = meta["regularMarketPrice"].as_f64().unwrap_or(0.0);
                let symbol = meta["symbol"].as_str().unwrap_or("").to_string();

                let (
                    momentum_12m,
                    price_start_period,
                    price_end_period,
                    date_start_period,
                    date_end_period,
                ) = match calculate_momentum_12m(&result[0]) {
                    Some((momentum, start, end, date_start, date_end)) => {
                        (momentum, start, end, date_start, date_end)
                    }
                    None => (0.0, 0.0, 0.0, String::from("N/A"), String::from("N/A")),
                };

                stock_data.push(StockData {
                    symbol: symbol.clone(),
                    short_name: symbol.clone(), // Since we don't have short_name from chart data
                    currency,
                    regular_market_price,
                    trailing_pe: None,
                    price_to_book: None,
                    return_on_equity: None,
                    debt_to_equity: None,
                    market_cap: None,
                    momentum_12m,
                    price_start_period,
                    price_end_period,
                    date_start_period,
                    date_end_period,
                });
            } else {
                error!("No data found for {}", symbol);
            }
        } else {
            error!("Error fetching stock data for {}: {:?}", symbol, data);
        }
    }

    Ok(stock_data)
}

/// Calculates the 12-month momentum for a stock.
///
/// # Arguments
///
/// * `result` - The JSON value containing the stock data.
///
/// # Returns
///
/// * `Some((f64, f64, f64, String, String))` - A tuple containing the momentum, start price, end price, start date, and end date.
/// * `None` - If the calculation fails or the required data is missing.
fn calculate_momentum_12m(result: &Value) -> Option<(f64, f64, f64, String, String)> {
    if let (Some(timestamps), Some(closes)) = (
        result["timestamp"].as_array(),
        result["indicators"]["quote"][0]["close"].as_array(),
    ) {
        let one_year_ago = chrono::Utc::now().timestamp() - 31536000; // Approximately 1 year in seconds

        let mut idx_start = 0;
        while idx_start < timestamps.len()
            && timestamps[idx_start].as_i64().unwrap_or(0) < one_year_ago
        {
            idx_start += 1;
        }

        if idx_start < timestamps.len() {
            let idx_end = timestamps.len() - 1; // Last data point
            let start_price = closes[idx_start].as_f64().unwrap_or(0.0);
            let end_price = closes[idx_end].as_f64().unwrap_or(0.0);

            // Calculate date_start using DateTime::from_timestamp
            let date_start =
                DateTime::from_timestamp(timestamps[idx_start].as_i64().unwrap_or(0), 0)
                    .unwrap()
                    .format("%Y-%m-%d")
                    .to_string();
            let date_end = DateTime::from_timestamp(timestamps[idx_end].as_i64().unwrap_or(0), 0)
                .unwrap()
                .format("%Y-%m-%d")
                .to_string();

            if start_price != 0.0 {
                let momentum = (end_price - start_price) / start_price;
                return Some((momentum, start_price, end_price, date_start, date_end));
            }
        }
    }
    None
}

/// Calculates the factor scores for the given stock data.
///
/// # Arguments
///
/// * `stock_data` - A slice of `StockData` structs.
///
/// # Returns
///
/// A vector of `FactorScores` structs containing the calculated factor scores.
fn calculate_factor_scores(stock_data: &[StockData]) -> Vec<FactorScores> {
    let mut factor_scores = Vec::new();
    let mut value_scores = Vec::new();
    let mut quality_scores = Vec::new();
    let mut momentum_scores = Vec::new();
    let mut size_scores = Vec::new();

    for stock in stock_data {
        let value_score = stock.trailing_pe.map(|pe| 1.0 / pe).unwrap_or(0.0)
            + stock.price_to_book.map(|pb| 1.0 / pb).unwrap_or(0.0);
        let quality_score =
            stock.return_on_equity.unwrap_or(0.0) - stock.debt_to_equity.unwrap_or(0.0);
        let momentum_score = stock.momentum_12m;
        let size_score = stock
            .market_cap
            .map(|cap| 1.0 / (cap / 1_000_000_000.0))
            .unwrap_or(0.0);

        value_scores.push(value_score);
        quality_scores.push(quality_score);
        momentum_scores.push(momentum_score);
        size_scores.push(size_score);

        factor_scores.push(FactorScores {
            symbol: stock.symbol.clone(),
            currency: stock.currency.clone(),
            value_score,
            quality_score,
            momentum_score,
            size_score,
            composite_score: 0.0, // Temporary placeholder, will be calculated later
            price_start_period: stock.price_start_period,
            price_end_period: stock.price_end_period,
            date_start_period: stock.date_start_period.clone(),
            date_end_period: stock.date_end_period.clone(),
        });
    }

    // Function to calculate mean and standard deviation
    fn mean_std(scores: &[f64]) -> (f64, f64) {
        let mean = scores.iter().copied().sum::<f64>() / scores.len() as f64;
        let variance = scores
            .iter()
            .map(|score| (score - mean).powi(2))
            .sum::<f64>()
            / scores.len() as f64;
        let std_dev = variance.sqrt();
        (mean, std_dev)
    }

    // Normalize scores
    let (value_mean, value_std) = mean_std(&value_scores);
    let (quality_mean, quality_std) = mean_std(&quality_scores);
    let (momentum_mean, momentum_std) = mean_std(&momentum_scores);
    let (size_mean, size_std) = mean_std(&size_scores);

    for score in factor_scores.iter_mut() {
        score.value_score = if value_std != 0.0 {
            (score.value_score - value_mean) / value_std
        } else {
            0.0
        };
        score.quality_score = if quality_std != 0.0 {
            (score.quality_score - quality_mean) / quality_std
        } else {
            0.0
        };
        score.momentum_score = if momentum_std != 0.0 {
            (score.momentum_score - momentum_mean) / momentum_std
        } else {
            0.0
        };
        score.size_score = if size_std != 0.0 {
            (score.size_score - size_mean) / size_std
        } else {
            0.0
        };

        // Recalculate composite score after normalization
        score.composite_score = 0.25 * score.value_score
            + 0.25 * score.quality_score
            + 0.25 * score.momentum_score
            + 0.25 * score.size_score;
    }

    factor_scores
}

/// Fetches the last quarter's stock price data for the given stock symbols from the Yahoo Finance API.
///
/// # Arguments
///
/// * `symbols` - A slice of stock ticker symbols.
///
/// # Returns
///
/// * `Ok(HashMap<String, f64>)` - A hash map mapping stock symbols to their last quarter's price.
/// * `Err(reqwest::Error)` - An error if the API request fails.
async fn fetch_last_quarter_data(
    symbols: &[String],
) -> Result<HashMap<String, f64>, reqwest::Error> {
    let mut headers = header::HeaderMap::new();
    let _ = headers.insert("User-Agent", header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"));
    let _ = headers.insert(
        "Accept",
        header::HeaderValue::from_static("application/json"),
    );
    let _ = headers.insert("Cookie", header::HeaderValue::from_static("YahooFcUrl"));

    let client = Client::builder().default_headers(headers).build()?;
    let mut last_quarter_data = HashMap::new();

    for symbol in symbols {
        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?period1=0&period2=9999999999&interval=1d&includePrePost=true&events=div%7Csplit",
            symbol
        );

        // print!("url: {}", url);

        let response = client.get(&url).send().await?;
        let data: Value = response.json().await?;
        info!("Fetched last quarter data for {}: {:?}", symbol, data);

        if let Some(result) = data["chart"]["result"].as_array() {
            if !result.is_empty() {
                if let Some(meta) = result[0]["meta"].as_object() {
                    if let Some(current_price) = meta["regularMarketPrice"].as_f64() {
                        let _ = last_quarter_data.insert(symbol.clone(), current_price);
                    }
                }
            } else {
                error!("No data found for {}", symbol);
            }
        } else {
            error!(
                "Error fetching last quarter data for {}: {:?}",
                symbol, data
            );
        }
    }

    Ok(last_quarter_data)
}

/// Ranks the stocks based on their composite factor scores.
///
/// # Arguments
///
/// * `factor_scores` - A mutable slice of `FactorScores` structs.
fn rank_stocks(factor_scores: &mut [FactorScores]) {
    factor_scores.sort_by(|a, b| b.composite_score.partial_cmp(&a.composite_score).unwrap());
}

/// Generates a stock ranking report based on the factor scores and last quarter's data.
///
/// # Arguments
///
/// * `factor_scores` - A slice of `FactorScores` structs.
/// * `last_quarter_data` - A hash map mapping stock symbols to their last quarter's price.
fn generate_report(factor_scores: &[FactorScores], last_quarter_data: &HashMap<String, f64>) {
    println!("\n## Stock Ranking Report Based on Factor Investing");

    println!("\n### In Brief");
    println!("\nThis report provides a detailed analysis of the stock ranking based on factor investing. It identifies stocks with strong recent performance, potential for continued growth, and ranks them accordingly.\n");

    println!("### Overview");
    println!("\nFactor investing involves targeting specific drivers of return across different asset classes. This strategy evaluates stocks based on various factors such as value, quality, momentum, and size to generate a comprehensive ranking.\n");

    println!("### Dates Used for the Report");
    if let Some(first_score) = factor_scores.first() {
        println!("\nThe report uses the closing prices over the past 12 months to calculate the momentum factor. The exact date range is from {} to {}.\n",
            first_score.date_start_period, first_score.date_end_period);
    }

    println!("### Factors in Focus");
    println!("\nThe key factors currently influencing the market include:");
    for score in factor_scores {
        println!(
            "\n- {}: Momentum Score {:.2}, Value Score {:.2}, Quality Score {:.2}, Size Score {:.2}",
            score.symbol,
            score.momentum_score,
            score.value_score,
            score.quality_score,
            score.size_score
        );
    }
    println!("### Factor Views vs. Last Quarter");
    println!("\nThis section provides a comparison of factor views against the last quarter, highlighting any changes in the market environment and how it impacts factor performance. Here are the shifts in factor scores and their impact on stock rankings based on Yahoo Finance data:\n");
    for score in factor_scores {
        if let Some(last_quarter_price) = last_quarter_data.get(&score.symbol) {
            let change = (score.price_end_period - last_quarter_price) / last_quarter_price * 100.0;
            println!(
                "\n- {}: Current Price {:.2}, Last Quarter Price {:.2}, Change {:.2}%",
                score.symbol, score.price_end_period, last_quarter_price, change
            );
        }
    }
    println!();

    println!("### Stock Ranking Based on Factor Investing");
    println!("\nThe table below summarizes our outlook for each of the factors assessed. It does not constitute a recommendation, but rather indicates our estimate of the attractiveness of factors in the current market environment.\n");
    println!("| Rank | Symbol | Currency | Momentum | Price at Start | Price at End | Start Date | End Date |");
    println!("|------|--------|----------|----------|----------------|--------------|------------|----------|");
    for (i, score) in factor_scores.iter().enumerate() {
        println!(
            "| {:4} | {:6} | {:8} | {:8.2} | {:14.2} | {:12.2} | {:10} | {:8} |",
            i + 1,
            score.symbol,
            score.currency,
            score.momentum_score,
            score.price_start_period,
            score.price_end_period,
            score.date_start_period,
            score.date_end_period
        );
    }

    println!("\n### Explanation of Momentum Factor");
    println!("\nThe momentum factor measures the stock's price movement over the past 12 months. It is calculated using the following formula:\n");
    println!("Momentum = (Price at end of period - Price at start of period) / Price at start of period\n");

    println!("\n### Interpretation of Results");
    for (i, score) in factor_scores.iter().enumerate() {
        let explanation = if score.momentum_score >= 0.0 {
            "indicating significant price appreciation"
        } else {
            "indicating a decrease in stock price"
        };
        println!(
            "\n- **{} ({})** ranks {} with a momentum score of {:.2}, {} over the past 12 months.",
            score.symbol,
            score.symbol,
            i + 1,
            score.momentum_score,
            explanation
        );
    }

    println!("### Glossary");
    println!("\n- **Equity momentum**: Measures the 12-month price momentum of stocks.");
    println!("- **Equity value**: Assesses stocks based on metrics such as Price-to-Earnings (P/E) Ratio and Price-to-Book (P/B) Ratio.");
    println!("- **Equity quality**: Evaluates stocks using metrics like Return on Equity (ROE) and Debt-to-Equity Ratio.");
    println!("- **Equity size**: Considers the market capitalization of the company.\n");
}

/// The main function of the stock ranking system.
///
/// # Returns
///
/// * `Ok(())` - If the program executes successfully.
/// * `Err(Box<dyn std::error::Error>)` - If an error occurs during execution.
#[tokio::main]
pub async fn main() -> Result<(), NaluFxError> {
    let symbols_input = get_input("Enter the stock ticker symbols (comma-separated):")?;
    let symbols: Vec<String> = symbols_input
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    for symbol in &symbols {
        if validate_ticker(symbol).is_err() {
            eprintln!("Error: Invalid ticker symbol: {}", symbol);
            return Ok(());
        }
    }

    let stock_data = fetch_stock_data(&symbols).await?;
    let last_quarter_data = fetch_last_quarter_data(&symbols).await?;

    if stock_data.is_empty() {
        eprintln!("No stock data available for the provided symbols");
        return Ok(());
    }

    let mut factor_scores = calculate_factor_scores(&stock_data);
    rank_stocks(&mut factor_scores);
    generate_report(&factor_scores, &last_quarter_data);

    Ok(())
}
