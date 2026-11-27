<img src="https://kura.pro/nalufx/images/logos/nalufx.svg" alt="NaluFX (NFX) logo" height="66" width="66" align="right" />

# NaluFX (NFX)

A Rust library and HTTP service that forecasts fund cash flows and allocates cash across a forecast horizon. Every number it returns is deterministic: the same input gives the same output, and anything it cannot compute is an explicit error.

[![Made With Love][made-with-rust]][05]
[![License][license-badge]][02]

## Important Notice

1. **Early stage:** NaluFX is in early development. Its interfaces change between releases until v0.1.0.

2. **Research and analysis only:** NaluFX's forecasts and allocations are for research and analysis. They are not investment recommendations or financial advice. Consult a qualified financial advisor before making investment decisions.

## What it does

- **Allocation across assets** (`nalufx-engine`), each method long-only with a budget of 1, a per-asset cap and an optional floor, and each result recording its method, parameters and solver diagnostics:
  - `equal_weight`: the baseline every other method must beat.
  - `min_variance`: the least-variance portfolio, a quadratic programme solved with [clarabel][11].
  - `mean_variance`: Markowitz, `mu' w - (lambda / 2) w' S w`, with the risk aversion as a parameter. It leans on estimated mean returns, which are noisy, so out of sample it often trails equal weight (DeMiguel, Garlappi and Uppal, 2009); on NaluFX's backtest fixture its Sharpe ratio is 1.49 against equal weight's 1.52, and the report shows that rather than tuning it away.
  - `risk_parity`: equal risk contribution, by Newton's method on the convex log-barrier formulation.
  - `hrp`: hierarchical risk parity, with no matrix inversion, for near-singular covariances.
- **Covariance:** sample, or Ledoit-Wolf shrinkage towards a scaled identity (the default).
- **Backtests:** walk-forward, rebalancing on a schedule, net of a cost in basis points, reporting annualised return, volatility, Sharpe, maximum drawdown and turnover.
- **Cash flow forecasting:** AutoETS (via [augurs][10]) for a plain series, or MSTL for weekly, monthly or other cycles; every forecast carries its 95% prediction interval.
- **Screening:** keep the assets whose score (an ESG rating you supply, for example) meets a threshold. NaluFX has no rating data of its own.
- **HTTP API:** `POST /predict` returns the forecast and an allocation by the method you name; errors are RFC 9457 problem details (`application/problem+json`).
- **Deterministic throughout:** the same input gives the same output, bit for bit, and anything that cannot be computed is an explicit error. A 20-asset, 250-day allocation takes well under a millisecond.
- **Reports:** example reports that put the computed numbers in front of a language model for commentary. The model writes words only; it never produces a figure NaluFX reports.

## Installation

NaluFX is not on crates.io yet. Depend on it from Git:

```toml
[dependencies]
nalufx = { git = "https://github.com/sebastienrousseau/nalufx", tag = "v0.0.6", package = "nalufx-core" }
```

Or install the HTTP service:

```shell
cargo install --git https://github.com/sebastienrousseau/nalufx --tag v0.0.6 nalufx-core
```

### Requirements

`nalufx` requires Rust **1.92** or later.

### Documentation

Build the API documentation locally with `cargo doc --open -p nalufx-core`. More is on the [website][00].

## Usage

### Allocating across assets

```rust
use nalufx_engine::allocate::{by_name, AllocationInput, Constraints};
use nalufx_engine::covariance::CovarianceEstimator;
use ndarray::array;

let input = AllocationInput {
    assets: vec!["BONDS".into(), "STOCKS".into(), "GOLD".into()],
    // One row per day, one column per asset.
    returns: array![
        [0.001, 0.012, -0.004], [0.002, -0.010, 0.006], [-0.001, 0.015, 0.001],
        [0.001, -0.004, -0.007], [0.000, 0.008, 0.002], [0.001, -0.006, 0.004],
    ],
    constraints: Constraints { max_weight: 0.6, min_weight: 0.05 },
    estimator: CovarianceEstimator::LedoitWolf,
};
let allocation = by_name("min_variance", None)?.allocate(&input)?;
println!("{allocation}");
# Ok::<(), nalufx_engine::AllocationError>(())
```

### Backtesting a method

```rust
use nalufx_engine::allocate::{Constraints, Hrp};
use nalufx_engine::backtest::{run, BacktestConfig};
use nalufx_engine::covariance::CovarianceEstimator;

# let assets: Vec<String> = vec!["A".into(), "B".into()];
# let returns = ndarray::Array2::from_shape_fn((300, 2), |(i, j)| 0.01 * ((i * (j + 3)) as f64).sin());
let config = BacktestConfig { lookback: 120, rebalance_every: 21, cost_bps: 10.0, periods_per_year: 252.0 };
let report = run(&Hrp, &assets, &returns, Constraints::default(), CovarianceEstimator::LedoitWolf, config)?;
println!("Sharpe {:.2}, max drawdown {:.1}%", report.sharpe, report.max_drawdown * 100.0);
# Ok::<(), nalufx_engine::AllocationError>(())
```

### The HTTP API

```shell
curl -s localhost:8080/predict -H 'content-type: application/json' -d '{
  "historical_data": [1000, 1012, 1009, 1021, 1030, 1026, 1041, 1038, 1050, 1047],
  "assets": ["BONDS", "STOCKS"],
  "returns": [[0.001, 0.012], [0.002, -0.010], [-0.001, 0.015], [0.001, -0.004], [0.000, 0.008], [0.001, -0.006]],
  "method": "risk_parity",
  "constraints": { "max_weight": 0.8 }
}'
```

A success is `200` with `{"forecast": {"level", "point", "lower", "upper"}, "allocation": {"assets", "weights", "method", "params", "diagnostics"}}`. `method` is one of `equal_weight`, `min_variance` (the default), `mean_variance` (with an optional `risk_aversion`), `risk_parity` and `hrp`; `estimator` is `ledoit_wolf` (the default) or `sample`; add `"seasonal_periods": [5]` to forecast a weekly cycle with MSTL. Input that cannot be used (empty, NaN, a ragged returns matrix, an unknown method) is `400`; valid input that cannot be forecast or solved (bounds no weights can meet, for example) is `422`.

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
[11]: https://clarabel.org

[license-badge]: https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue?style=for-the-badge 'License'
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust 'Made With Rust'
