//! `nalufx tui`: choose tickers and a method, run `allocate` or
//! `forecast` as the commands do, and read the result in a table. Built
//! on petitcarnets, a Rust port of Bubble Tea.

mod job;
mod keys;
mod model;
mod view;

pub use job::{run as run_job, Job, Outcome, Request, Source};
pub use model::{App, Choice, Msg, Screen, Ticker};

use crate::cli::TuiArgs;
use crate::error::CliError;

/// `nalufx tui`: the interface until q or Ctrl-C, then the terminal as
/// it was. A screen reader user sets `ACCESSIBLE=1` for plain lines.
///
/// # Errors
///
/// [`CliError::Input`] if the file cannot be read, before the interface
/// starts; [`CliError::Runtime`] if the terminal cannot be used.
pub fn run(args: &TuiArgs) -> Result<(), CliError> {
    let app = App::new(Source::from_args(&args.source)?, args.bounds, args.risk_aversion);
    petitcarnets::Program::new(app).run().map(drop).map_err(|e| CliError::Runtime {
        message: e.to_string(),
        hint: Some("run `nalufx tui` in a terminal".into()),
    })
}
