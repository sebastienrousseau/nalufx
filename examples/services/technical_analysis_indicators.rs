//! Technical Analysis Indicators
//!
//! This example demonstrates how to fetch historical stock data, calculate technical indicators,
//! and generate a comprehensive technical analysis report for a user-specified stock using OpenAI.
//! The code fetches historical closing prices for the stock, calculates various technical indicators,
//! and sends the data to the OpenAI API to generate interpretations and recommendations.
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
use nalufx::api::handlers::{get_openai_api_key, send_openai_request};
use nalufx::errors::NaluFxError;
use nalufx::services::fetch_data::fetch_data;
use nalufx::utils::input::get_input;
use serde_json::json;

/// Calculates the relative strength index (RSI) for the given data and window size.
///
/// # Arguments
///
/// * `data` - The slice of price data.
/// * `window` - The window size for the RSI calculation.
///
/// # Returns
///
/// A vector containing the RSI values.
fn calculate_rsi(data: &[f64], window: usize) -> Vec<f64> {
    let mut rsi = Vec::with_capacity(data.len());
    let mut gains = 0.0;
    let mut losses = 0.0;

    // Calculate initial gains and losses
    for i in 1..=window {
        let change = data[i] - data[i - 1];
        if change > 0.0 {
            gains += change;
        } else {
            losses -= change;
        }
    }

    rsi.push(100.0 - (100.0 / (1.0 + (gains / losses))));

    // Calculate the rest of the RSI values
    for i in (window + 1)..data.len() {
        let change = data[i] - data[i - 1];
        if change > 0.0 {
            gains = (gains * (window as f64 - 1.0) + change) / window as f64;
            losses = (losses * (window as f64 - 1.0)) / window as f64;
        } else {
            gains = (gains * (window as f64 - 1.0)) / window as f64;
            losses = (losses * (window as f64 - 1.0) - change) / window as f64;
        }

        rsi.push(100.0 - (100.0 / (1.0 + (gains / losses))));
    }

    rsi
}

// Function to validate if the input is a valid date in the format YYYY-MM-DD
fn validate_date(input: &str) -> Result<chrono::DateTime<Utc>, &str> {
    match NaiveDate::parse_from_str(input, "%Y-%m-%d") {
        Ok(date) => Ok(Utc
            .from_local_datetime(&date.and_hms_opt(0, 0, 0).unwrap())
            .unwrap()),
        _ => Err("Please enter a valid date in the format YYYY-MM-DD."),
    }
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

/// Calculates the moving average convergence divergence (MACD) for the given data.
///
/// # Arguments
///
/// * `data` - The slice of price data.
/// * `short_window` - The short window size for the MACD calculation.
/// * `long_window` - The long window size for the MACD calculation.
/// * `signal_window` - The window size for the signal line calculation.
///
/// # Returns
///
/// A tuple containing the MACD values, signal line values, and histogram values.
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

    let histogram: Vec<f64> = macd
        .iter()
        .zip(signal.iter())
        .map(|(macd_val, signal_val)| macd_val - signal_val)
        .collect();

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

/// Generates a professional technical analysis report using the OpenAI API.
///
/// # Arguments
///
/// * `closing_prices` - The slice of historical closing prices.
/// * `ema` - The calculated Exponential Moving Average (EMA) values.
/// * `rsi` - The calculated Relative Strength Index (RSI) values.
/// * `macd` - The calculated Moving Average Convergence Divergence (MACD) values.
/// * `macd_signal` - The calculated MACD signal values.
/// * `macd_histogram` - The calculated MACD histogram values.
/// * `support_levels` - The identified support levels.
/// * `resistance_levels` - The identified resistance levels.
///
/// Returns the generated report as a string.
async fn generate_technical_analysis_report(
    closing_prices: &[f64],
    ema: &[f64],
    rsi: &[f64],
    macd: &[f64],
    macd_signal: &[f64],
    macd_histogram: &[f64],
    support_levels: &[f64],
    resistance_levels: &[f64],
) -> Result<String, &'static str> {
    let client = reqwest::Client::new();
    let api_key = match get_openai_api_key() {
        Ok(key) => key,
        Err(err) => {
            eprintln!("{}", err);
            return Err("Failed to get OpenAI API key");
        }
    };

    let request_body = json!({
        "model": "gpt-3.5-turbo",
        "messages": [
            {
                "role": "system",
                "content": "You are a highly skilled financial analyst working for a reputable investment firm. Your task is to generate a comprehensive technical analysis report for a portfolio of stocks. The report should be written in a professional tone, similar to reports published by Bloomberg or other leading financial institutions. Provide detailed data-driven insights, quantitative analysis, and actionable recommendations. Please use the following structure:"
            },
            {
                "role": "user",
                "content": format!(
                    "
1. **Executive Summary:** Provide a concise summary of the key findings and recommendations.

2. **Market Overview:**
    * Analyze the overall market and economic conditions during the analysis period.
    * Discuss relevant macroeconomic factors, industry trends, and geopolitical events.

3. **Portfolio Performance:**
    * Closing Prices: {:?}
    * EMA Values: {:?}
    * RSI Values: {:?}
    * MACD Values: {:?}
    * MACD Signal: {:?}
    * MACD Histogram: {:?}
    * Analyze the performance of each stock in the portfolio, including closing prices, trend analysis (based on EMA), momentum analysis (based on RSI), and convergence/divergence analysis (based on MACD). 
    * Explicitly mention the calculated values for each indicator.

4. **Risk Assessment:**
    * Support Levels: {:?}
    * Resistance Levels: {:?}
    * Identify potential support and resistance levels for each stock.
    * Discuss the implications for risk management strategies, such as setting appropriate stop-loss and take-profit levels.

5. **Investment Strategies:**
    * Based on the technical analysis, provide recommendations for the following investment strategies:
        * **Flexible Income Strategy:** For investors seeking a flexible income stream (e.g., using drawdown income) at their target retirement age and beyond.
        * **Annuity Purchase Strategy:** For investors aiming to purchase a regular income (annuity) from an insurance company at their target retirement age and beyond.
        * **Lump Sum Strategy:** For investors seeking one or two cash lump sums at their target retirement age and beyond.
        * For each strategy, outline specific stock recommendations, entry/exit points, and risk management considerations.

6. **Disclaimer:**
    * Emphasize that this report is for informational purposes only and does not constitute financial advice.
    * Encourage readers to conduct their own research and consult with financial advisors before making investment decisions.

Please ensure that the report is well-structured, easy to understand, and adheres to industry-standard formatting and terminology.
                ", closing_prices, ema, rsi, macd, macd_signal, macd_histogram, support_levels, resistance_levels
                )
            }
        ],
        "max_tokens": 1500,
    });

    let openai_url = "https://api.openai.com/v1/chat/completions";
    let response = match send_openai_request(&client, openai_url, &api_key, request_body).await {
        Ok(response) => response,
        Err(err) => return Err(err),
    };

    let openai_response: nalufx::api::handlers::OpenAIResponse = serde_json::from_str(&response)
        .map_err(|err| {
            eprintln!("Error parsing response JSON: {:?}", err);
            "Error parsing response JSON"
        })?;

    let generated_text = openai_response
        .choices
        .first()
        .and_then(|choice| Some(choice.message.content.clone()))
        .ok_or("No content found in response")?;

    Ok(generated_text)
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
pub(crate) async fn main() -> Result<(), NaluFxError> {
    // Get user input for ticker, initial investment amount, start date, and end date
    let ticker_input = get_input("Enter the ticker symbol for a bellwether stock:")?;
    let ticker = match validate_ticker(&ticker_input) {
        Ok(symbol) => symbol.to_string(),
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(NaluFxError::InvalidOption);
        }
    };

    // Fetch historical price data for the stock
    let start_date_input = get_input("Enter the start date (YYYY-MM-DD):")?;
    let start_date = match validate_date(&start_date_input) {
        Ok(date) => date,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(NaluFxError::InvalidOption);
        }
    };

    let end_date_input = get_input("Enter the end date (YYYY-MM-DD):")?;
    let end_date = match validate_date(&end_date_input) {
        Ok(date) => date,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(NaluFxError::InvalidOption);
        }
    };

    let closing_prices = match fetch_data(&ticker, Some(start_date), Some(end_date)).await {
        Ok(prices) => prices,
        Err(e) => {
            eprintln!("Error fetching historical data: {}", e);
            return Err(NaluFxError::ForecastingError(
                "Failed to fetch historical data".to_string(),
            )
            .into());
        }
    };

    // Calculate technical indicators
    let ema_window = 50;
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
    let report = match generate_technical_analysis_report(
        &closing_prices,
        &ema,
        &rsi,
        &macd,
        &macd_signal,
        &macd_histogram,
        &support_levels,
        &resistance_levels,
    )
    .await
    {
        Ok(report) => report,
        Err(err) => {
            eprintln!("Error: {}", err);
            return Err(NaluFxError::ReinforcementLearningError(
                "Failed to generate technical analysis report".to_string(),
            )
            .into());
        }
    };

    println!("\n--- Professional Technical Analysis Report ---\n");
    println!("Ticker: {}", ticker);
    println!(
        "Period: {} to {}",
        start_date.format("%Y-%m-%d"),
        end_date.format("%Y-%m-%d")
    );

    // Print the data sections
    println!("\n--- Price Analysis ---\n");
    println!("Closing Prices: {:?}", closing_prices);
    if support_levels.is_empty() {
        println!("Support Levels: No support levels identified in the provided data.");
    } else {
        println!("Support Levels: {:?}", support_levels);
    }
    if resistance_levels.is_empty() {
        println!("Resistance Levels: No resistance levels identified in the provided data.");
    } else {
        println!("Resistance Levels: {:?}", resistance_levels);
    }

    println!("\n--- Trend Analysis ---\n");
    println!("Exponential Moving Average (EMA) - Window: {}", ema_window);
    println!("EMA Values: {:?}", ema);

    println!("\n--- Momentum Analysis ---\n");
    println!("Relative Strength Index (RSI) - Window: {}", rsi_window);
    println!("RSI Values: {:?}", rsi);

    println!("\n--- Convergence/Divergence Analysis ---\n");
    println!(
        "Moving Average Convergence Divergence (MACD) - Short Window: {}, Long Window: {}, Signal Window: {}",
        macd_short_window, macd_long_window, macd_signal_window
    );
    println!("MACD Values: {:?}", macd);
    println!("MACD Signal: {:?}", macd_signal);
    println!("MACD Histogram: {:?}", macd_histogram);

    // Print the OpenAI-generated report
    println!("\n--- Advanced Technical Analysis Interpretations ---\n");
    println!("{}", report);

    Ok(())
}
