//! # nalufx-cli
//!
//! The `nalufx` command: allocate, forecast, backtest and report, from
//! live prices or CSV files, with a table for people and `--json` for
//! scripts. Run `nalufx --help`.

pub mod cli;
pub mod commands;
pub mod error;
pub mod input;
pub mod settings;
pub mod tui;
