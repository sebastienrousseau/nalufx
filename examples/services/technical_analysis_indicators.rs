//! Technical Analysis Indicators
//!
//! This example demonstrates how to fetch historical stock data, calculate technical indicators,
//! and generate a comprehensive technical analysis report for a user-specified stock.
//! The code fetches historical closing prices for the stock, calculates various technical indicators,
//! and provides interpretations based on the indicator values.
//! The results are presented in a detailed report format.
//!
//! Usage:
//! 1. Run the code using `cargo run --example technical_analysis_indicators`.
//! 2. Enter the ticker symbol for the desired stock when prompted.
//! 3. The code will fetch historical data for the stock, calculate technical indicators, and generate a report with the analysis.
//!
//! Technical Indicators:
//! - Exponential Moving Average (EMA): Calculates the exponentially weighted average price over a specified number of periods. It gives more weight to recent prices, making it more responsive to price changes. It helps identify the overall trend direction.
//! - Relative Strength Index (RSI): Measures the magnitude of recent price changes to evaluate overbought or oversold conditions. It ranges from 0 to 100.
//! - Moving Average Convergence Divergence (MACD): Consists of MACD line, signal line, and histogram. It helps identify trend changes and momentum.
//! - Support and Resistance Levels: Represents key price levels where the stock tends to find support or resistance. They are used to identify potential entry and exit points.

use chrono::NaiveDate;
use chrono::{TimeZone, Utc};
use std::io;

use nalufx::errors::NaluFxError;
use nalufx::services::fetch_data::fetch_data;

/// Calculates the simple moving average (SMA) for the given data and window size.
///
/// # Arguments
///
/// * `data` - The slice of price data.
/// * `window` - The window size for the SMA calculation.
///
/// # Returns
///
/// A vector of SMA values.
fn calculate_sma(data: &[f64], window: usize) -> Vec<f64> {
    data.windows(window)
        .map(|slice| slice.iter().sum::<f64>() / slice.len() as f64)
        .collect()
}

/// Calculates the relative strength index (RSI) for the given data and window size.
///
/// # Arguments
///
/// * `data` - The slice of price data.
/// * `window` - The window size for the RSI calculation.
///
/// # Returns
///
/// A vector of RSI values.
fn calculate_rsi(data: &[f64], window: usize) -> Vec<f64> {
    let gains: Vec<f64> = data
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).max(0.0))
        .collect();
    let losses: Vec<f64> = data
        .windows(2)
        .map(|pair| (pair[0] - pair[1]).max(0.0))
        .collect();

    let avg_gain = calculate_sma(&gains, window);
    let avg_loss = calculate_sma(&losses, window);

    let rs: Vec<f64> = avg_gain
        .iter()
        .zip(avg_loss.iter())
        .map(|(g, l)| g / l)
        .collect();
    rs.into_iter()
        .map(|rs| 100.0 - (100.0 / (1.0 + rs)))
        .collect()
}

/// Prompts the user for input and returns the user's response.
///
/// # Arguments
///
/// * `prompt` - The prompt message to display to the user.
///
/// # Returns
///
/// The user's input as a string.
fn get_input(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string()
}

/// Validates if the input is non-empty and alphanumeric.
///
/// # Arguments
///
/// * `input` - The input string to validate.
///
/// # Returns
///
/// `Ok(input)` if the input is valid, `Err` with an error message otherwise.
fn validate_ticker(input: &str) -> Result<&str, &str> {
    if input.chars().all(|c| c.is_alphanumeric()) && !input.is_empty() {
        Ok(input)
    } else {
        Err("Please enter a valid ticker symbol (alphanumeric).")
    }
}

/// Calculates the exponential moving average (EMA) for the given data and window size.
///
/// # Arguments
///
/// * `data` - The slice of price data.
/// * `window` - The window size for the EMA calculation.
///
/// # Returns
///
/// A vector of EMA values.
///
/// The EMA calculation gives more weight to recent data points, making it more responsive to recent price changes.
/// The EMA is calculated using the following formula:
///
/// ```
/// EMA = (Close - EMA(prev)) * Multiplier + EMA(prev)
/// ```
///
/// Where:
/// - `Close` is the current closing price
/// - `EMA(prev)` is the previous EMA value
/// - `Multiplier` is a smoothing factor calculated as `2 / (window + 1)`
fn calculate_ema(data: &[f64], window: usize) -> Vec<f64> {
    let mut ema = vec![data[0]];
    let multiplier = 2.0 / (window as f64 + 1.0);

    for i in 1..data.len() {
        let current_ema = (data[i] - ema[i - 1]) * multiplier + ema[i - 1];
        ema.push(current_ema);
    }

    ema
}

/// Calculates the moving average convergence divergence (MACD) for the given data and window sizes.
///
/// # Arguments
///
/// * `data` - The slice of price data.
/// * `short_window` - The short window size for the MACD calculation.
/// * `long_window` - The long window size for the MACD calculation.
/// * `signal_window` - The signal window size for the MACD calculation.
///
/// # Returns
///
/// A tuple containing the MACD values, signal values, and histogram values.
fn calculate_macd(
    data: &[f64],
    short_window: usize,
    long_window: usize,
    signal_window: usize,
) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let short_ema = calculate_ema(data, short_window);
    let long_ema = calculate_ema(data, long_window);

    let macd: Vec<f64> = short_ema
        .iter()
        .zip(long_ema.iter())
        .map(|(short, long)| short - long)
        .collect();

    let signal = calculate_ema(&macd, signal_window);

    let histogram: Vec<f64> = macd.iter().zip(signal.iter()).map(|(m, s)| m - s).collect();

    (macd, signal, histogram)
}

/// Identifies support and resistance levels for the given data and window size.
///
/// # Arguments
///
/// * `data` - The slice of price data.
/// * `window` - The window size for identifying support and resistance levels.
///
/// # Returns
///
/// A tuple containing the support levels and resistance levels.
fn identify_support_resistance(data: &[f64], window: usize) -> (Vec<f64>, Vec<f64>) {
    let mut support = Vec::new();
    let mut resistance = Vec::new();

    for i in window..data.len() - window {
        let slice = &data[i - window..i + window + 1];
        let min = slice
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let max = slice
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        if data[i] == *min {
            support.push(data[i]);
        } else if data[i] == *max {
            resistance.push(data[i]);
        }
    }

    (support, resistance)
}

/// The main function that drives the execution of the technical analysis report generation.
///
/// This function handles the following tasks:
///
/// 1. Prompts the user to enter the stock ticker symbol.
/// 2. Prompts the user to enter the start and end dates for the historical data period.
/// 3. Fetches the historical closing prices for the specified stock and date range.
/// 4. Calculates various technical indicators, including:
///     - Exponential Moving Average (EMA)
///     - Relative Strength Index (RSI)
///     - Moving Average Convergence Divergence (MACD)
///     - Support and Resistance Levels
/// 5. Generates a comprehensive technical analysis report with the calculated indicator values and interpretations.
/// 6. Provides specific recommendations based on the technical analysis.
/// 7. Includes risk management guidelines and a disclaimer.
///
/// # Returns
///
/// A `Result` indicating whether the execution was successful or not. If successful, returns `Ok(())`, otherwise returns an error wrapped in a `Box<dyn std::error::Error>`.
#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get user input for the stock ticker symbol
    let ticker = loop {
        let input = get_input("Enter the ticker symbol for the stock:");
        match validate_ticker(&input) {
            Ok(ticker) => break ticker.to_string(),
            Err(e) => eprintln!("Error: {}", e),
        }
    };

    // Fetch historical price data for the stock
    let start_date = loop {
        let input = get_input("Enter the start date (YYYY-MM-DD):");
        match NaiveDate::parse_from_str(&input, "%Y-%m-%d") {
            Ok(date) => {
                let start_datetime = Utc
                    .from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
                    .unwrap();
                break Some(start_datetime);
            }
            Err(_) => eprintln!("Error: Invalid start date format. Please use YYYY-MM-DD."),
        }
    };

    let end_date = loop {
        let input = get_input("Enter the end date (YYYY-MM-DD) or leave blank for today:");
        if input.is_empty() {
            break Some(Utc::now());
        } else {
            match NaiveDate::parse_from_str(&input, "%Y-%m-%d") {
                Ok(date) => {
                    let end_datetime = Utc
                        .from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
                        .unwrap();
                    break Some(end_datetime);
                }
                Err(_) => eprintln!("Error: Invalid end date format. Please use YYYY-MM-DD."),
            }
        }
    };

    let closing_prices = match fetch_data(&ticker, start_date, end_date).await {
        Ok(prices) => prices,
        Err(e) => {
            eprintln!("Error fetching historical data: {}", e);
            return Err(NaluFxError::new("Failed to fetch historical data").into());
        }
    };

    // Calculate technical indicators
    let ema_window = 50; // Change the window size as desired
    let rsi_window = 14;
    let macd_short_window = 12;
    let macd_long_window = 26;
    let macd_signal_window = 9;
    let support_resistance_window = 10;

    let ema = calculate_ema(&closing_prices, ema_window);
    let rsi = calculate_rsi(&closing_prices, rsi_window);
    let (macd, macd_signal, macd_histogram) = calculate_macd(
        &closing_prices,
        macd_short_window,
        macd_long_window,
        macd_signal_window,
    );
    let (support_levels, resistance_levels) =
        identify_support_resistance(&closing_prices, support_resistance_window);

    // Generate the professional technical analysis report
    println!("\n--- Professional Technical Analysis Report ---");
    println!("Ticker: {}", ticker);
    println!(
        "Period: {} to {}",
        start_date.unwrap().format("%Y-%m-%d"),
        end_date.unwrap().format("%Y-%m-%d")
    );
    println!("\n--- Price Analysis ---");
    println!("Closing Prices: {:?}", closing_prices);
    println!("Support Levels: {:?}", support_levels);
    println!("Resistance Levels: {:?}", resistance_levels);

    println!("\n--- Trend Analysis ---");
    println!("Exponential Moving Average (EMA) - Window: {}", ema_window);
    println!("EMA Values: {:?}", ema);

    println!("\n--- Momentum Analysis ---");
    println!("Relative Strength Index (RSI) - Window: {}", rsi_window);
    println!("RSI Values: {:?}", rsi);

    println!("\n--- Convergence/Divergence Analysis ---");
    println!(
        "Moving Average Convergence Divergence (MACD) - Short Window: {}, Long Window: {}, Signal Window: {}",
        macd_short_window, macd_long_window, macd_signal_window
    );
    println!("MACD Values: {:?}", macd);
    println!("MACD Signal: {:?}", macd_signal);
    println!("MACD Histogram: {:?}", macd_histogram);

    // Provide advanced technical analysis interpretations
    println!("\n--- Advanced Technical Analysis Interpretations ---");
    println!("Trend Interpretation:");
    if let (Some(&last_price), Some(&last_sma), Some(&last_ema)) =
        (closing_prices.last(), ema.last(), ema.last())
    {
        if last_price > last_sma && last_price > last_ema {
            println!(
                "- The stock is in a strong bullish trend, with the price above both SMA and EMA."
            );
        } else if last_price < last_sma && last_price < last_ema {
            println!(
                "- The stock is in a strong bearish trend, with the price below both SMA and EMA."
            );
        } else {
            println!("- The stock is in a neutral trend, with mixed signals from SMA and EMA.");
        }
    }

    println!("\nMomentum Interpretation:");
    if let Some(&last_rsi) = rsi.last() {
        if last_rsi > 70.0 {
            println!("- The RSI is above 70, indicating that the stock is overbought and may be due for a price correction.");
        } else if last_rsi < 30.0 {
            println!("- The RSI is below 30, indicating that the stock is oversold and may be due for a price rebound.");
        } else {
            println!("- The RSI is between 30 and 70, suggesting neutral momentum.");
        }
    }

    println!("\nConvergence/Divergence Interpretation:");
    if let (Some(&last_macd), Some(&last_signal)) = (macd.last(), macd_signal.last()) {
        if last_macd > last_signal {
            println!("- The MACD is above the signal line, indicating a bullish trend.");
        } else if last_macd < last_signal {
            println!("- The MACD is below the signal line, indicating a bearish trend.");
        } else {
            println!(
                "- The MACD and signal line are converging, suggesting a potential trend change."
            );
        }
    }

    // Provide specific recommendations based on the technical analysis
    println!("\n--- Professional Recommendations ---");
    if let (Some(&last_price), Some(&last_sma), Some(&last_rsi)) =
        (closing_prices.last(), ema.last(), rsi.last())
    {
        if last_price > last_sma && last_rsi < 70.0 {
            println!("- Strong Buy: The stock is in a bullish trend and has a healthy momentum. Consider opening a long position.");
        } else if last_price < last_sma && last_rsi > 30.0 {
            println!("- Strong Sell: The stock is in a bearish trend and has a weak momentum. Consider opening a short position.");
        } else {
            println!("- Hold: The technical indicators suggest a neutral outlook. Monitor the stock for further developments.");
        }
    }

    println!("\n--- Risk Management ---");
    println!("- Set appropriate stop-loss and take-profit levels based on the identified support and resistance levels.");
    println!("- Diversify your portfolio to manage overall risk exposure.");
    println!("- Regularly monitor the stock's performance and adjust your positions accordingly.");

    println!("\n--- Disclaimer ---");
    println!("This technical analysis report is provided for informational purposes only and should not be considered as investment advice.");
    println!("The analysis is based on historical data and past performance, which is not indicative of future results.");
    println!("Always conduct thorough research, consider your investment objectives, and consult with a professional financial advisor before making any investment decisions.");
    println!("The authors and providers of this report shall not be held liable for any investment decisions made based on the information provided.");

    Ok(())
}
