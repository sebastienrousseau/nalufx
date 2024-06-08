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
- AI-driven suggestions for cash allocation optimization
- Clustering using K-means for data segmentation
- Data fetching from multiple sources
- Data processing and transformation utilities
- Reinforcement learning for dynamic decision-making
- Sentiment analysis
- Structured and easy-to-parse output formats
- Time series forecasting

## Installation

To use `nalufx` in your Rust project, add the following to your `Cargo.toml` file:

```toml
[dependencies]
nalufx = "0.0.1"
```

### Requirements

`nalufx` requires Rust **1.56** or later.

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

NaluFX comes with a set of examples that you can use to get started. The examples are located in the `examples` directory of the project.

To run the examples, clone the repository and run the following command in your terminal from the project root directory:

```shell
cargo run --example nalufx
```

You can also run the examples directly from the repository by using the following commands:

```shell
cargo run --example <example_name>
```

### Bellwether Stock Analysis

Analyze the performance of a bellwether stock during a significant historical period (e.g., the 1987 market crash).

```shell
cargo run --example bellwether_stock_analysis
```

### Diversified ETF Portfolio Optimization

Optimize a portfolio of diversified ETFs based on historical performance and risk factors.

```shell
cargo run --example diversified_etf_portfolio_optimization
```

### ESG Portfolio Optimization

Optimize a portfolio with a strong emphasis on environmental, social, and governance (ESG) factors.

```shell
cargo run --example esg_portfolio_optimization
```

### Factor Investing Stock Ranking

Rank stocks based on various investing factors such as value, momentum, and quality.

```shell
cargo run --example factor_investing_stock_ranking
```

### Fetch Data Example

Demonstrate how to fetch financial data from an API or a data source.

```shell
cargo run --example fetch_stock_data
```

### Generate Market Analysis Report

Generate a comprehensive market analysis report based on various financial metrics and indicators.

```shell
cargo run --example generate_portfolio_report
```

### Mean Variance Optimization

Implement mean-variance optimization to find the optimal risk-return tradeoff for a portfolio.

```shell
cargo run --example mean_variance_optimization
```

### Risk Parity Portfolio Optimization

Optimize a portfolio using the risk parity approach to ensure balanced risk distribution.

```shell
cargo run --example risk_parity_portfolio_optimization
```

### Technical Analysis Indicators

Implement and analyze various technical analysis indicators such as moving averages, RSI, MACD, etc.

```shell
cargo run --example technical_analysis_indicators
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
