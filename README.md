<img src="https://kura.pro/nalufx/images/logos/nalufx.svg" alt="NaluFX (NFX) logo" height="66" width="66" align="right" />

# NaluFX (NFX)

A Rust library and HTTP service that forecasts fund cash flows and allocates cash across a forecast horizon. Every number it returns is deterministic: the same input gives the same output, and anything it cannot compute is an explicit error.

[![Made With Love][made-with-rust]][05]
[![License][license-badge]][02]

## Important Notice

1. **Early stage:** NaluFX is in early development. Its interfaces change between releases until v0.1.0.

2. **Research and analysis only:** NaluFX's forecasts and allocations are for research and analysis. They are not investment recommendations or financial advice. Consult a qualified financial advisor before making investment decisions.

## What it does

- **Cash flow forecasting:** AutoETS (automatic exponential smoothing, via [augurs][10]) picks the best non-seasonal model by AICc and returns a point forecast with its 95% prediction interval.
- **Allocation across a horizon:** the investment is spread over the next days by inverse volatility, each day weighted by the inverse of its forecast interval's width. Weights are long-only and sum to 1. This is the interim method; a mean-variance optimiser replaces it in a later release.
- **HTTP API:** `POST /predict` returns the forecast and the allocation; errors are RFC 9457 problem details (`application/problem+json`).
- **Deterministic clustering:** K-means with farthest-point seeding, for segmenting feature rows.
- **Reports:** example reports that put the computed numbers in front of a language model for commentary. The model writes words only; it never produces a figure NaluFX reports.

## Installation

NaluFX is not on crates.io yet. Depend on it from Git:

```toml
[dependencies]
nalufx = { git = "https://github.com/sebastienrousseau/nalufx", tag = "v0.0.5", package = "nalufx-core" }
```

Or install the HTTP service:

```shell
cargo install --git https://github.com/sebastienrousseau/nalufx --tag v0.0.5 nalufx-core
```

### Requirements

`nalufx` requires Rust **1.92** or later.

### Documentation

Build the API documentation locally with `cargo doc --open -p nalufx-core`. More is on the [website][00].

## Usage

### Forecasting and allocating

```rust
use nalufx::utils::calculations::{calculate_optimal_allocation, forecast_time_series};

let daily_returns = [0.01, -0.02, 0.015, 0.003, -0.007, 0.012, -0.004, 0.009, -0.011, 0.006];

// A 3-day forecast with its 95% interval.
let forecast = forecast_time_series(&daily_returns, 3)?;
println!("{:?} in [{:?}, {:?}]", forecast.point, forecast.lower, forecast.upper);

// The same 3 days, weighted by inverse forecast volatility.
let weights = calculate_optimal_allocation(&daily_returns, 3)?;
assert!((weights.iter().sum::<f64>() - 1.0).abs() < 1e-9);
# Ok::<(), nalufx::errors::AllocationError>(())
```

### The HTTP API

```shell
curl -s localhost:8080/predict -H 'content-type: application/json' -d '{
  "historical_data": [1000, 1012, 1009, 1021, 1030, 1026, 1041, 1038, 1050, 1047],
  "daily_returns":   [0.01, -0.02, 0.015, 0.003, -0.007, 0.012, -0.004, 0.009, -0.011, 0.006]
}'
```

A success is `200` with `{"forecast": {"level", "point", "lower", "upper"}, "optimal_allocation": [...]}` for the next 6 days. Input that cannot be used (empty, NaN, a daily return over 100%) is `400`; valid input that cannot be forecast or weighted (for example a constant series) is `422`.

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

### Automate Cash Allocation

Automatically allocate cash to ETFs and mutual funds based on historical performance and market conditions.

```shell
cargo run --example automate_cash_allocation
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
[10]: https://github.com/grafana/augurs

[license-badge]: https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=for-the-badge 'License'
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust 'Made With Rust'
