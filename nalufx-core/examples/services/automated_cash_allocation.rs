//! # Automated Cash Allocation Example
//!
//! This example demonstrates the automation of cash allocation to ETFs and Mutual Funds
//! using predefined allocation rules. It fetches the latest fund data, applies allocation
//! rules, executes the allocation orders, generates a report, and prints the results to the console.
//!
//! ## Usage
//!
//! 1. Ensure you have the necessary data files:
//!    - `data/etf_data.csv` for ETF data
//!    - `data/mutual_fund_data.csv` for Mutual Fund data
//!    - `data/allocation_rules.json` for allocation rules
//! 2. Run the code using `cargo run --example automated_cash_allocation`.
//! 3. The code will automatically process the data and display the allocation results.
//!
//! The generated report will be saved to `data/allocation_report.json`.

use csv::Reader;
use nalufx_llms::llms::{LLM, openai, openai::OpenAI};
use nalufx::{
    errors::NaluFxError,
    services::automated_cash_allocation_svc::generate_analysis,
    utils::{currency::format_currency, date::validate_date, input::get_input}
};
use reqwest::{header, Client};
use serde::Serialize;
use std::{collections::HashMap, io::BufReader};
use tokio::{fs, io::AsyncReadExt};

use nalufx::models::allocation_dm::{AllocationOrder, Etf, MutualFund, AllocationRules};

/// Represents a report of allocation orders.
#[derive(Debug, Serialize)]
struct Report {
    etf_orders: Vec<AllocationOrder>,
    mutual_fund_orders: Vec<AllocationOrder>,
    total_allocation: f64,
    analysis: String,
}

/// The main function for the automated cash allocation example.
#[tokio::main]
pub(crate) async fn main() -> Result<(), NaluFxError> {
    // Get user input for LLM choice
    let llm_choice = get_input("Enter the LLM to use (e.g., openai, claude, gemini, llama, mistral, ollama):")?;
    let (llm, api_key): (Box<dyn LLM>, String) = match llm_choice.as_str() {
        "openai" => {
            let api_key = match openai::get_openai_api_key() {
                Ok(key) => key,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    return Err(NaluFxError::InvalidData);
                }
            };
            (Box::new(OpenAI), api_key)
        },
        // Add other cases for different LLMs with their respective API key functions
        _ => {
            eprintln!("Unsupported LLM choice");
            return Err(NaluFxError::InvalidOption);
        }
    };

    // Get user input for portfolio name, investor's values, and financial objectives
    let portfolio_name = get_input("Enter the name of the portfolio - (e.g., Growth Portfolio, Balanced Portfolio, Sustainable Future Portfolio):")?;
    let values_input = get_input("Enter the investor's values (comma-separated) - (e.g., Environmental sustainability, social responsibility, corporate governance):")?;
    let financial_objectives_input = get_input("Enter the investor's financial objectives (comma-separated) - (e.g., Long-term capital appreciation, moderate risk tolerance):")?;

    let start_date_input = get_input("Enter the start date (YYYY-MM-DD):")?;
    let _start_date = match validate_date(&start_date_input) {
        Ok(date) => date,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(NaluFxError::InvalidOption);
        }
    };

    let end_date_input = get_input("Enter the end date (YYYY-MM-DD):")?;
    let _end_date = match validate_date(&end_date_input) {
        Ok(date) => date,
        Err(e) => {
            eprintln!("Error: {}", e);
            return Err(NaluFxError::InvalidOption);
        }
    };

    // Step 1: Fetch latest fund data
    let etf_data = fetch_etf_data("data/etf_data.csv").await?;
    let mutual_fund_data = fetch_mutual_fund_data("data/mutual_fund_data.csv").await?;

    // Step 2: Determine allocation percentages
    let allocation_rules = load_allocation_rules("data/allocation_rules.json").await?;
    let mut etf_allocation = allocate_funds(&etf_data, allocation_rules.etf_percentage);
    let mut mutual_fund_allocation = allocate_funds(&mutual_fund_data, allocation_rules.mutual_fund_percentage);

    // Step 3: Fetch real-time prices for all symbols
    let all_symbols: Vec<String> = etf_data
        .iter()
        .map(|etf| etf.symbol.clone())
        .chain(mutual_fund_data.iter().map(|mf| mf.symbol.clone()))
        .collect();
    let real_time_prices = fetch_real_time_prices(&all_symbols).await?;

    // Update prices in allocations
    update_prices_in_allocations(&mut etf_allocation, &real_time_prices);
    update_prices_in_allocations(&mut mutual_fund_allocation, &real_time_prices);

    // Step 4: Generate detailed analysis
    let client = Client::new();
    let analysis = generate_analysis(
        llm, // Pass the boxed trait object here
        &client,
        &api_key,
        &portfolio_name,
        &etf_allocation,
        &mutual_fund_allocation,
        &values_input,
        &financial_objectives_input,
        &start_date_input,
        &end_date_input,
        &real_time_prices,
    )
    .await
    .map_err(|e| {
        eprintln!("Error generating analysis: {}", e);
        NaluFxError::InvalidData
    })?;

    // Step 5: Generate report
    let report = generate_allocation_report(&etf_allocation, &mutual_fund_allocation, analysis);
    save_report(&report, "data/allocation_report.json").await?;

    // Print results dynamically in the console
    print_results(&report);

    Ok(())
}

/// Fetches ETF data from a CSV file.
async fn fetch_etf_data(file_path: &str) -> Result<Vec<Etf>, NaluFxError> {
    let file = fs::File::open(file_path).await.map_err(|e| {
        NaluFxError::InputError(std::io::Error::new(
            e.kind(),
            format!("Failed to open ETF data file: {}", file_path),
        ))
    })?;
    let std_file = file.into_std().await;
    let mut rdr = Reader::from_reader(BufReader::new(std_file));
    let mut etfs = Vec::new();
    for result in rdr.deserialize() {
        let etf: Etf = result?;
        etfs.push(etf);
    }
    Ok(etfs)
}

/// Fetches Mutual Fund data from a CSV file.
async fn fetch_mutual_fund_data(file_path: &str) -> Result<Vec<MutualFund>, NaluFxError> {
    let file = fs::File::open(file_path).await.map_err(|e| {
        NaluFxError::InputError(std::io::Error::new(
            e.kind(),
            format!("Failed to open Mutual Fund data file: {}", file_path),
        ))
    })?;
    let std_file = file.into_std().await;
    let mut rdr = Reader::from_reader(BufReader::new(std_file));
    let mut funds = Vec::new();
    for result in rdr.deserialize() {
        let fund: MutualFund = result?;
        funds.push(fund);
    }
    Ok(funds)
}

/// Loads allocation rules from a JSON file.
async fn load_allocation_rules(file_path: &str) -> Result<AllocationRules, NaluFxError> {
    let mut file = fs::File::open(file_path).await.map_err(|e| {
        NaluFxError::InputError(std::io::Error::new(
            e.kind(),
            format!("Failed to open allocation rules file: {}", file_path),
        ))
    })?;
    let mut data = String::new();
    let _ = file.read_to_string(&mut data).await?;
    let rules: AllocationRules = serde_json::from_str(&data)?;
    Ok(rules)
}

/// Allocates funds according to the provided allocation rules.
fn allocate_funds<T>(fund_data: &[T], percentage: f64) -> Vec<AllocationOrder>
where
    T: FundData,
{
    let total_value: f64 = fund_data.iter().map(|fund| fund.value()).sum();
    let allocation_amount = total_value * (percentage / 100.0);

    fund_data
        .iter()
        .map(|fund| {
            let amount = allocation_amount * (fund.value() / total_value);
            AllocationOrder {
                symbol: fund.symbol().to_string(),
                name: fund.name().to_string(),
                amount,
            }
        })
        .collect()
}

/// Updates allocation orders with real-time prices.
fn update_prices_in_allocations(
    allocations: &mut [AllocationOrder],
    prices: &HashMap<String, (f64, f64)>,
) {
    for allocation in allocations.iter_mut() {
        if let Some((_, current_price)) = prices.get(&allocation.symbol) {
            allocation.amount = *current_price;
        }
    }
}

/// Generates an allocation report.
fn generate_allocation_report(
    etf_allocation: &[AllocationOrder],
    mutual_fund_allocation: &[AllocationOrder],
    analysis: String,
) -> Report {
    let total_allocation: f64 = etf_allocation
        .iter()
        .chain(mutual_fund_allocation.iter())
        .map(|order| order.amount)
        .sum();
    Report {
        etf_orders: etf_allocation.to_vec(),
        mutual_fund_orders: mutual_fund_allocation.to_vec(),
        total_allocation,
        analysis,
    }
}

/// Prints the results of the allocation report.
fn print_results(report: &Report) {
    println!("\n--- Allocation Report ---");
    println!(
        "\nTotal Allocation: {}",
        format_currency(report.total_allocation)
    );
    println!("\nETF Orders:\n");
    for order in &report.etf_orders {
        println!(
            "Name: {}, Symbol: {}, Amount: {}",
            order.name,
            order.symbol,
            format_currency(order.amount)
        );
    }
    println!("\nMutual Fund Orders:\n");
    for order in &report.mutual_fund_orders {
        println!(
            "Name: {}, Symbol: {}, Amount: {}",
            order.name,
            order.symbol,
            format_currency(order.amount)
        );
    }
    println!(
        "\n--- Automated Cash Allocation Analysis ---\n\n{}",
        report.analysis
    );
}

/// Saves the allocation report to a JSON file.
async fn save_report(report: &Report, file_path: &str) -> Result<(), NaluFxError> {
    let file = fs::File::create(file_path).await.map_err(|e| {
        NaluFxError::InputError(std::io::Error::new(
            e.kind(),
            format!("Failed to create report file: {}", file_path),
        ))
    })?;
    let std_file = file.into_std().await;
    let writer = std::io::BufWriter::new(std_file);
    serde_json::to_writer_pretty(writer, report).map_err(|_e| {
        NaluFxError::InvalidData
    })?;
    Ok(())
}

/// Trait to define common behaviour for fund data.
trait FundData {
    fn symbol(&self) -> &str;
    fn name(&self) -> &str;
    fn value(&self) -> f64;
}

impl FundData for Etf {
    fn symbol(&self) -> &str {
        &self.symbol
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn value(&self) -> f64 {
        self.price * self.shares_outstanding
    }
}

impl FundData for MutualFund {
    fn symbol(&self) -> &str {
        &self.symbol
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn value(&self) -> f64 {
        self.price * self.net_assets
    }
}

/// Fetches real-time price data from Yahoo Finance for the given symbols.
async fn fetch_real_time_prices(
    symbols: &[String],
) -> Result<HashMap<String, (f64, f64)>, reqwest::Error> {
    let mut headers = header::HeaderMap::new();
    let _ = headers.insert("User-Agent", header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"));
    let _ = headers.insert("Accept", header::HeaderValue::from_static("application/json"));
    let _ = headers.insert("Cookie", header::HeaderValue::from_static("YahooFcUrl"));

    let client = Client::builder().default_headers(headers).build()?;
    let mut prices = HashMap::new();

    for symbol in symbols {
        let url = format!(
            "https://query1.finance.yahoo.com/v8/finance/chart/{}?period1=0&period2=9999999999&interval=1d&includePrePost=true&events=div%7Csplit",
            symbol
        );

        let response = client.get(&url).send().await?;
        let data: serde_json::Value = response.json().await?;
        if let Some(result) = data["chart"]["result"].as_array() {
            if let Some(_timestamps) = result.get(0).and_then(|r| r["timestamp"].as_array()) {
                if let Some(closes) = result.get(0).and_then(|r| r["indicators"]["quote"][0]["close"].as_array()) {
                    if let (Some(start_price), Some(end_price)) = (closes.first().and_then(|v| v.as_f64()), closes.last().and_then(|v| v.as_f64())) {
                        let _ = prices.insert(symbol.clone(), (start_price, end_price));
                    }
                }
            }
        }
    }

    Ok(prices)
}
