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
- **HTTP API** (`nalufx-api`, on axum and tower): `POST /v1/forecast`, `POST /v1/allocate` and `POST /v1/report` behind bearer API keys (stored as SHA-256 hashes, compared in constant time) and per-key rate limits, with open `GET /`, `GET /openapi.json`, `GET /healthz` and `GET /readyz`. A 30 s timeout, a concurrency limit, a 1 MiB body limit, compression, CORS denied by default, graceful shutdown on SIGTERM, and RFC 9457 problem details with a stable type URI and the request id on every error. The contract is OpenAPI 3.1, generated from the code and committed as [`nalufx-api/openapi.json`](nalufx-api/openapi.json).
- **Deterministic throughout:** the same input gives the same output, bit for bit, and anything that cannot be computed is an explicit error. A 20-asset, 250-day allocation takes well under a millisecond.
- **Narrative reports** (`nalufx-llms`): one `LlmProvider` trait with native Anthropic (Messages API) and Gemini (`generateContent`) adapters and one OpenAI-compatible adapter for OpenAI, Mistral, Ollama, Agy and Llama servers. The model is always yours to name; there is no default. Requests share one client with 5 s connect and 60 s request timeouts, and a timeout, `429` or `5xx` is retried up to 3 times with exponential backoff and jitter, honouring `Retry-After`. The model is given the computed figures and a JSON Schema, writes words only, and a reply that does not match the schema is an error, never text. Prompts are versioned templates in `nalufx-llms/prompts/`, and every report records the prompt, provider and model that wrote it.

## Installation

NaluFX is not on crates.io yet. Depend on it from Git:

```toml
[dependencies]
nalufx = { git = "https://github.com/sebastienrousseau/nalufx", tag = "v0.0.11", package = "nalufx-core" }
```

Or install the command line and the HTTP server:

```shell
cargo install --git https://github.com/sebastienrousseau/nalufx --tag v0.0.11 nalufx-cli   # the `nalufx` command
cargo install --git https://github.com/sebastienrousseau/nalufx --tag v0.0.11 nalufx-api   # the HTTP server
```

### Requirements

`nalufx` requires Rust **1.92** or later.

### Documentation

Build the API documentation locally with `cargo doc --open -p nalufx-core`. More is on the [website][00].

## Usage

### The command line

`nalufx` runs every part of the engine from a terminal, from live prices or CSV files, with a table for people and `--json` for scripts. `nalufx --help`, and `--help` on any command, lists every option. These work offline, on the synthetic data in `nalufx-cli/data`:

```shell
nalufx allocate --file nalufx-cli/data/sample-returns.csv --method risk-parity --max-weight 0.4
nalufx backtest --file nalufx-cli/data/sample-returns.csv              # every method side by side
nalufx forecast --file nalufx-cli/data/sample-cash.csv --column net_cash_flow --seasonal 5
nalufx methods                                                         # what each method does
nalufx providers                                                       # which LLM keys are set
```

With a network connection, `--assets SPY,TLT,GLD` fetches daily prices instead of `--file`. A report pipes an allocation to a language model, which writes the narrative:

```shell
nalufx allocate --assets SPY,TLT,GLD --json \
  | nalufx report --provider anthropic --model claude-sonnet-5-5 --from -
```

Bad input is caught before anything slow happens: bounds no weights can meet before prices are fetched, a missing API key before the input is read. Every error says what to do about it. The exit codes are `0` success, `2` usage, `3` unusable input, `4` no answer (such as bounds no weights can meet), `5` a network, price feed or language model failure, and `1` anything else.


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

### Writing a report

```rust,no_run
use nalufx_llms::{http::Http, provider::{ClientConfig, LlmConfig}, registry::from_env};
use nalufx_llms::report::{generate, ReportInput};
use serde_json::json;

# async fn run() -> Result<(), nalufx_llms::error::LlmError> {
// ANTHROPIC_API_KEY comes from the environment or a .env file.
let provider = from_env("anthropic", LlmConfig::new("claude-sonnet-5-5", 2048)?, Http::new(ClientConfig::default())?)?;
let report = generate(provider.as_ref(), &ReportInput {
    portfolio: "Treasury reserve".into(),
    facts: json!({ "method": "min_variance", "assets": ["BONDS", "STOCKS"], "weights": [0.82, 0.18] }),
    context: "Preserve capital; spend in six months.".into(),
}).await?;
println!("{} (prompt {} v{}, {})", report.report.summary, report.prompt.name, report.prompt.version, report.model);
# Ok(())
# }
```

### The HTTP API

Running it in production (configuration, key rotation, logs, metrics, alerts and the audit log) is in [docs/runbook.md](docs/runbook.md).

Start the server with at least one API key, stored as its SHA-256 hash:

```shell
export NALUFX_API_KEY_SHA256="$(printf %s "$MY_KEY" | sha256sum | cut -d' ' -f1)"
nalufx-api   # listens on 127.0.0.1:8080; NALUFX_ADDR changes it
```

Then call it with the key:

```shell
curl -s localhost:8080/v1/allocate -H "authorization: Bearer $MY_KEY" -H 'content-type: application/json' -d '{
  "assets": ["BONDS", "STOCKS"],
  "returns": [[0.001, 0.012], [0.002, -0.010], [-0.001, 0.015], [0.001, -0.004], [0.000, 0.008], [0.001, -0.006]],
  "method": "risk_parity",
  "constraints": { "max_weight": 0.8 }
}'
```

| Route | Key | Body | Answer |
| --- | --- | --- | --- |
| `POST /v1/forecast` | yes | `history`, optional `horizon_days` (1 to 366, default 6) and `seasonal_periods` | The point forecast with its 95% interval |
| `POST /v1/allocate` | yes | `assets`, `returns` (one row per period), optional `method`, `risk_aversion`, `constraints`, `estimator` | The weights, method, parameters and diagnostics |
| `POST /v1/report` | yes | `portfolio`, `facts` (the computed figures), `context` | A schema-checked narrative, with its prompt, provider and model |
| `GET /` | no | | Where the contract and the health routes are |
| `GET /openapi.json` | no | | The OpenAPI 3.1 contract (the committed `nalufx-api/openapi.json`) |
| `GET /healthz` | no | | `200` while the process is up |
| `GET /readyz` | no | | `200` when the configuration is loaded and the language model's endpoint answers |

`method` is `equal_weight`, `min_variance` (the default), `mean_variance`, `risk_parity` or `hrp`. Every error is `application/problem+json` with a `type` under `https://nalufx.com/problems/` and the `request_id`: `400` for unusable input, `401` without a valid key, `408` past the timeout, `413` over 1 MiB, `422` when the maths has no answer (bounds no weights can meet, for example), `429` over a key's rate with `Retry-After`, and `502`/`503`/`504` when the language model fails. `/v1/report` needs `NALUFX_LLM_PROVIDER` and `NALUFX_LLM_MODEL` (and the provider's key); without them it answers `503`. Every setting is listed in the `nalufx_api::config` documentation.

## Examples

The `nalufx-core/examples/services` directory has small programs that call the library directly, each prompting for its inputs. Run one with:

```shell
cargo run -p nalufx-core --example <example_name>
```

For day-to-day use, the `nalufx` command above is the interface; these show the library calls behind it.

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
