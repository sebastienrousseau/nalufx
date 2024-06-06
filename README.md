<img src="https://kura.pro/nalufx/images/logos/nalufx.svg" alt="NaluFX (NFX) logo" height="66" width="66" align="right" />

# NaluFX (NFX)

A Rust library designed to optimize cash allocation across complex fund structures using AI-driven predictions and suggestions. It implements advanced financial models, data fetching, and processing for enhanced cash flow management.

[![Made With Love][made-with-rust]][05]
[![Crates.io][crates-badge]][07]
[![Lib.rs][libs-badge]][09]
[![Docs.rs][docs-badge]][08]
[![License][license-badge]][02]

![divider][divider]

## Overview

NaluFX (NFX) is a Rust library that provides sophisticated tools for financial modelling, data fetching, and processing, with a focus on optimizing cash allocation across complex fund structures. It leverages AI-driven predictions and suggestions to enhance decision-making processes.

## Features

- Advanced cash flow prediction models
- Data fetching from multiple sources
- Data processing and transformation utilities
- Structured and easy-to-parse output formats
- Time series forecasting
- Sentiment analysis
- Reinforcement learning for dynamic decision-making
- Clustering using K-means for data segmentation
- AI-driven suggestions for cash allocation optimization

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

### AI-Driven Cash Allocation Suggestions

```rust
use nalufx::services::handlers::{AllocationRequest, AllocationResponse};

let allocation_request = AllocationRequest {
    fund_data: vec![/* Your fund data here */],
    market_conditions: vec![/* Current market conditions */],
};

let allocation_response: AllocationResponse = nalufx::handlers::suggest_allocations(allocation_request).unwrap();

println!("Allocation Suggestions: {:?}", allocation_response.suggestions);
```

## Examples

NaluFX comes with a set of examples that you can use to get started. The examples are located in the `examples` directory of the project. To run the examples, clone the repository and run the following command in your terminal from the project root directory:

```shell
cargo run --example nalufx
```

### Bellwether Stock Analysis (1987)

- Analyze the performance of a bellwether stock during a significant historical period (e.g., the 1987 market crash).

### Diversified ETF Portfolio Optimization

- Optimize a portfolio of diversified ETFs based on historical performance and risk factors.

### ESG Portfolio Optimization

- Optimize a portfolio with a strong emphasis on environmental, social, and governance (ESG) factors.

### Factor Investing Stock Ranking

- Rank stocks based on various investing factors such as value, momentum, and quality.

### Fetch Data Example

- Demonstrate how to fetch financial data from an API or a data source.

### Generate Market Analysis Report

- Generate a comprehensive market analysis report based on various financial metrics and indicators.

### Technical Analysis Indicators

- Implement and analyze various technical analysis indicators such as moving averages, RSI, MACD, etc.

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

[00]: https://nalufx.com
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
