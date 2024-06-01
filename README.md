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
use nalufx::fetch_data::fetch_cash_flow_data;

let data = fetch_cash_flow_data("some_data_source").unwrap();
```

### Processing Data

```rust
use nalufx::processing::process_cash_flow_data;

let processed_data = process_cash_flow_data(&data);
```

### Cash Flow Predictions

```rust
use nalufx::models::{CashFlowRequest, CashFlowResponse};

let request = CashFlowRequest {
    historical_data: vec![100.0, 200.0, 300.0],
};

let response: CashFlowResponse = nalufx::handlers::predict_cash_flows(request).unwrap();

println!("Predictions: {:?}", response.predictions);
```

## Configuration

By default, NaluFX (NFX) uses a set of built-in configurations. You can customize configurations by setting environment variables or modifying configuration files.

## Error Handling

Errors can occur during data fetching, processing, or prediction operations. The library uses Rust's `Result` type to indicate success or failure. You should handle potential errors appropriately in your code.

```rust
use nalufx::fetch_data::fetch_cash_flow_data;

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
