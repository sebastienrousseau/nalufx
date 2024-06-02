<img src="https://kura.pro/nalufx/images/logos/nalufx.svg"
alt="NaluFX (NFX) logo" height="66" width="66" align="right" />

# NaluFX (NFX)

A Rust library that implements financial models, data fetching, and processing for cash flow predictions with a simple, readable output format.

[![Made With Love][made-with-rust]][05]
[![Crates.io][crates-badge]][07]
[![Lib.rs][libs-badge]][09]
[![Docs.rs][docs-badge]][08]
[![License][license-badge]][02]

![divider][divider]

## Overview

NaluFX (NFX) is a Rust library that provides tools for financial modeling, data fetching, and processing, with a focus on cash flow predictions. It offers APIs and various helper macros to simplify common financial tasks.

## Features

- Cash flow prediction models
- Data fetching from multiple sources
- Data processing and transformation utilities
- Structured and easy-to-parse output formats
- Time series forecasting
- Sentiment analysis
- Reinforcement learning
- Clustering using K-means

## Installation

To use `nalufx` in your Rust project, add the following to your `Cargo.toml` file:

```toml
[dependencies]
nalufx = "0.0.1"
```

### Requirements

`nalufx` requires Rust **1.60** or later.

### Documentation

> ℹ️ **Info:** Please check out our [website][00] for more information and find our documentation on [docs.rs][08], [lib.rs][09], and [crates.io][07].

## Usage

### Fetching Data

```rust
use nalufx::services::fetch_data::fetch_cash_flow_data;

let data = fetch_cash_flow_data("some_data_source").unwrap();
```

### Processing Data

```rust
use nalufx::services::processing::process_cash_flow_data;

let processed_data = process_cash_flow_data(&data);
```

### Cash Flow Predictions

```rust
use nalufx::services::models::{CashFlowRequest, CashFlowResponse};

let request = CashFlowRequest {
    historical_data: vec![100.0, 200.0, 300.0],
};

let response: CashFlowResponse = nalufx::handlers::predict_cash_flows(request).unwrap();

println!("Predictions: {:?}", response.predictions);
```

### Optimal Allocation Calculation

```rust
use nalufx::utils::calculations::calculate_optimal_allocation;

let daily_returns = vec![0.01, 0.02, -0.01, 0.03];
let cash_flows = vec![100.0, 200.0, 150.0, 250.0];
let market_indices = vec![1000.0, 1010.0, 1005.0, 1015.0];
let fund_characteristics = vec![0.8, 0.9, 0.85, 0.95];
let num_days = 4;

match calculate_optimal_allocation(&daily_returns, &cash_flows, &market_indices, &fund_characteristics, num_days) {
    Ok(result) => println!("Optimal allocation: {:?}", result),
    Err(e) => eprintln!("Error calculating optimal allocation: {}", e),
}
```

### Clustering

```rust
use ndarray::array;
use nalufx::utils::calculations::perform_clustering;

let features = array![[0.01, 100.0], [0.02, 200.0], [-0.01, 150.0], [0.03, 250.0]];
match perform_clustering(&features) {
    Ok(clusters) => println!("Clusters: {:?}", clusters),
    Err(e) => eprintln!("Error performing clustering: {}", e),
}
```

### Example Script

Here's an example of fetching historical data, calculating optimal allocation, and printing the recommendations.

```rust
use chrono::{TimeZone, Utc};
use nalufx::{
    services::{
        fetch_data::fetch_data,
        processing::{calculate_cash_flows, calculate_daily_returns},
    },
    utils::calculations::calculate_optimal_allocation,
};

// Custom function to format float as currency
fn format_currency(value: f64) -> String {
    let int_value = (value * 100.0).round() as i64; // Convert to integer cents
    let dollars = int_value / 100;
    let cents = (int_value % 100).abs(); // Absolute value for cents
    let formatted_dollars = format_dollars(dollars);
    format!("${}.{:02}", formatted_dollars, cents)
}

// Helper function to format the dollar amount with commas
fn format_dollars(dollars: i64) -> String {
    let mut s = dollars.to_string();
    let len = s.len();
    if len > 3 {
        let mut pos = len % 3;
        if pos == 0 {
            pos = 3;
        }
        while pos < len {
            s.insert(pos, ',');
            pos += 4;
        }
    }
    s
}

#[tokio::main]
pub(crate) async fn main() {
    // Define the ticker symbol and initial investment amount
    let ticker = "SPY";
    let initial_investment = 100000.0;

    // Fetch historical closing prices for SPY
    match fetch_data(ticker, None, None).await {
        Ok(closes) => {
            // Calculate daily returns from closing prices
            let daily_returns = calculate_daily_returns(&closes);

            // Calculate cash flows based on daily returns and initial investment
            let cash_flows = calculate_cash_flows(&daily_returns, initial_investment);

            // Fetch or generate market indices data
            let market_indices = vec![1000.0, 1010.0, 1005.0, 1015.0]; // Replace with actual data
            println!("\n--- Market Overview ---\n");
            println!(
                "The Market Indices represent key points of market performance during the period:\n"
            );
            for (i, value) in market_indices.iter().enumerate() {
                println!("- Index Point {}: {}", i + 1, format_currency(*value));
            }

            // Fetch or generate fund characteristics data
            let fund_characteristics = vec![0.8, 0.9, 0.85, 0.95]; // Replace with actual data
            println!(
                "\nThe Fund Characteristics represent key attributes of the fund during the period:\n"
            );
            for (i, value) in fund_characteristics.iter().enumerate() {
                println!("- Attribute {}: {:.2}", i + 1, value);
            }

            // Determine the minimum length of all input slices
            let min_length = daily_returns
                .len()
                .min(cash_flows.len())
                .min(market_indices.len())
                .min(fund_characteristics.len());

            // Truncate all slices to the minimum length
            let daily_returns = &daily_returns[..min_length];
            let cash_flows = &cash_flows[..min_length];
            let market_indices = &market_indices[..min_length];
            let fund_characteristics = &fund_characteristics[..min_length];

            // Calculate the optimal allocation based on truncated input slices
            let optimal_allocation_result = calculate_optimal_allocation(
                daily_returns,
                cash_flows,
                market_indices,
                fund_characteristics,
                min_length,
            );

            match optimal_allocation_result {
                Ok(mut optimal_allocation) => {
                    // Filter out negative allocations and normalize the rest
                    optimal_allocation = optimal_allocation
                        .into_iter()
                        .map(|alloc| if alloc < 0.0 { 0.0 } else { alloc })
                        .collect();
                    let total_allocation: f64 = optimal_allocation.iter().sum();
                    optimal_allocation = optimal_allocation
                        .into_iter()
                        .map(|alloc| alloc / total_allocation)
                        .collect();

                    // Print the optimal allocation with descriptive information
                    println!("\n--- Optimal Allocation Report ---\n");
                    println!(
                        "The optimal allocation represents the recommended distribution of funds for the next {} days.",
                        min_length
                    );
                    println!(
                        "Each value in the allocation vector corresponds to the percentage of funds to be allocated to {} for a specific day.",
                        ticker
                    );
                    println!("The sum of all values in the allocation vector should be close to 1.0 (100%).");
                    println!("\n- Optimal Allocation: {:?}", optimal_allocation);

                    // Provide specific recommendations based on the optimal allocation and initial investment
                    println!("\n--- Investment Recommendations ---\n");
                    println!(
                        "Based on the optimal allocation and your initial investment of {}, it is recommended to distribute your funds as follows:\n",
                        format_currency(initial_investment)
                    );

                    let today = Utc::now();
                    for (i, &allocation) in optimal_allocation.iter().enumerate() {
                        let allocation_amount = allocation * initial_investment;
                        let allocation_date = today + chrono::Duration::days(i as i64);
                        let allocation_percentage = allocation * 100.0;
                        println!(
                            "- Day {}: {} - Allocate {} ({:.2}%) to {}",
                            i + 1,
                            allocation_date.format("%Y-%m-%d"),
                            format_currency(allocation_amount),
                            allocation_percentage,
                            ticker
                        );
                    }



                    println!("\n--- Disclaimer ---\n");
                    println!("These recommendations are based on historical data and should be considered as a starting point for your investment strategy.");
                    println!("Market conditions can change rapidly, and past performance is not indicative of future results.");
                    println!("It is always advisable to conduct further research and consult with a financial advisor before making any investment decisions.\n");
                }
                Err(e) => {
                    eprintln!("Error calculating optimal allocation: {}", e);
                    println!("Please check the input data and try again.");
                }
            }
        }
        Err(e) => {
            eprintln!("Error fetching data for ticker {}: {}", ticker, e);
        }
    }
}
```

## Configuration

By default, NaluFX (NFX) uses a set of built-in configurations. You can customize configurations by setting environment variables or modifying configuration files.

## Error Handling

Errors can occur during data fetching, processing, or prediction operations. The library uses Rust's `Result` type to indicate success or failure. You should handle potential errors appropriately in your code.

```rust
use nalufx::services::fetch_data::fetch_cash_flow_data;

match fetch_cash_flow_data("some_data_source") {
    Ok(data) => println!("Data fetched successfully"),
    Err(e) => eprintln!("Error fetching data: {}", e),
}
```

## Examples

NaluFX comes with a set of examples that you can use to get started. The examples are located in the `examples` directory of the project. To run the examples, clone the repository and run the following command in your terminal from the project root directory:

```shell
cargo run --example nalufx
```

## Semantic Versioning Policy

For transparency into our release cycle and in striving to maintain backward compatibility, `NaluFX` follows [semantic versioning][06].

## License

The project is licensed under the terms of both the MIT license and the Apache License (Version 2.0).

- [Apache License, Version 2.0][01]
- [MIT license][02]

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## Acknowledgements

A special thank you goes to the [Rust Reddit](https://www.reddit.com/r/rust/) community for providing a lot of useful suggestions on how to improve this project.

[00]: https://rustlogs.com
[01]: http://www.apache.org/licenses/LICENSE-2.0
[02]: http://opensource.org/licenses/MIT
[05]: https://github.com/sebastienrousseau/nalufx/graphs/contributors
[06]: http://semver.org/
[07]: https://crates.io/crates/nalufx
[08]: https://docs.rs/nalufx
[09]: https://lib.rs/crates/nalufx

[crates-badge]: https://img.shields.io/crates/v/nalufx.svg?style=for-the-badge 'Crates.io'
[divider]: https://kura.pro/common/images/elements/divider.svg "divider"
[docs-badge]: https://img.shields.io/docsrs/nalufx.svg?style=for-the-badge 'Docs.rs'
[libs-badge]: https://img.shields.io/badge/lib.rs-v0.0.1-orange.svg?style=for-the-badge 'Lib.rs'
[license-badge]: https://img.shields.io/crates/l/nalufx.svg?style=for-the-badge 'License'
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust 'Made With Rust'
