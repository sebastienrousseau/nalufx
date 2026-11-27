//! `nalufx`: see `nalufx --help`.

use clap::Parser;
use nalufx_cli::cli::Cli;
use nalufx_cli::commands::execute;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    match execute(cli, &mut std::io::stdout().lock()).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            if let Some(hint) = err.hint() {
                eprintln!("  hint: {hint}");
            }
            ExitCode::from(err.exit_code())
        },
    }
}
