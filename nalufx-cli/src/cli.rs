//! The command line, as clap parses it.

use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// Deterministic forecasting and allocation from the command line.
///
/// Every figure is computed by the engine; a language model, if you use
/// `report`, only writes words about them. Research and analysis only;
/// not investment advice.
#[derive(Debug, Parser)]
#[command(name = "nalufx", version, about, long_about = None, arg_required_else_help = true)]
#[command(
    after_help = "Try it offline:\n  nalufx allocate --file nalufx-cli/data/sample-returns.csv\n  nalufx backtest --file nalufx-cli/data/sample-returns.csv --method all"
)]
pub struct Cli {
    /// A `nalufx.toml` to take settings from; `nalufx config check` shows
    /// the result. Environment variables and flags override it.
    #[arg(long, global = true, env = "NALUFX_CONFIG", value_name = "PATH")]
    pub config: Option<PathBuf>,
    /// What to do.
    #[command(subcommand)]
    pub command: Command,
}

/// The commands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Weights across assets by a method, within bounds.
    Allocate(AllocateArgs),
    /// A series' next days with a 95% prediction interval.
    Forecast(ForecastArgs),
    /// A walk-forward backtest of one method, or of all of them side by side.
    Backtest(BacktestArgs),
    /// A language model's narrative about an allocation (`allocate --json`).
    Report(ReportArgs),
    /// The allocation methods, with a line on each.
    Methods,
    /// The language model providers, and whether each one's key is set.
    Providers,
    /// The HTTP API server, until SIGTERM or Ctrl-C.
    Serve(ServeArgs),
    /// The configuration: `nalufx config check`.
    Config(ConfigArgs),
    /// An interactive allocate and forecast: choose tickers and a method,
    /// read the result in a table. `ACCESSIBLE=1` prints plain lines.
    Tui(TuiArgs),
}

/// `nalufx tui`.
#[derive(Debug, Args)]
#[command(
    after_help = "Try it offline:\n  nalufx tui --file nalufx-cli/data/sample-returns.csv\n  nalufx tui --assets SPY,TLT,GLD --max-weight 0.6"
)]
pub struct TuiArgs {
    /// Where the returns come from; the tickers to choose from are the
    /// file's columns or the tickers given.
    #[command(flatten)]
    pub source: ReturnsSource,
    /// Risk aversion, for mean-variance only.
    #[arg(long, value_parser = positive)]
    pub risk_aversion: Option<f64>,
    /// The bounds and estimator.
    #[command(flatten)]
    pub bounds: Bounds,
}

/// `nalufx serve`.
#[derive(Debug, Clone, Copy, Args)]
#[command(
    after_help = "Example:\n  NALUFX_API_KEY_SHA256=\"$(printf %s \"$KEY\" | sha256sum | cut -d' ' -f1)\" nalufx serve\n  nalufx serve --config nalufx.toml --addr 0.0.0.0:8080\n\ndocs/runbook.md covers every setting, the logs, metrics and alerts."
)]
pub struct ServeArgs {
    /// The listen address, over the file's and `NALUFX_ADDR`.
    #[arg(long, value_name = "HOST:PORT")]
    pub addr: Option<std::net::SocketAddr>,
}

/// `nalufx config`.
#[derive(Debug, Clone, Copy, Args)]
pub struct ConfigArgs {
    /// What to do with it.
    #[command(subcommand)]
    pub command: ConfigCommand,
}

/// The `nalufx config` commands.
#[derive(Debug, Clone, Copy, Subcommand)]
pub enum ConfigCommand {
    /// Checks the server's configuration and shows every setting, where it
    /// came from, and whether the provider's key is set; never a key.
    Check {
        /// Print JSON instead of a table.
        #[arg(long)]
        json: bool,
    },
}

/// Where the returns come from: live prices or a CSV file.
#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct ReturnsSource {
    /// Tickers to fetch daily prices for, comma-separated (needs the network).
    #[arg(long, value_delimiter = ',', value_name = "TICKERS")]
    pub assets: Vec<String>,
    /// A CSV of daily returns: a header of asset names, one row per day.
    #[arg(long, value_name = "PATH")]
    pub file: Option<PathBuf>,
}

/// The weight bounds and the covariance estimator.
#[derive(Debug, Args, Clone, Copy)]
pub struct Bounds {
    /// The largest weight any asset may have.
    #[arg(long, default_value_t = 1.0, value_parser = fraction)]
    pub max_weight: f64,
    /// The smallest weight every asset must have.
    #[arg(long, default_value_t = 0.0, value_parser = fraction)]
    pub min_weight: f64,
    /// How the covariance is estimated.
    #[arg(long, value_enum, default_value_t = Estimator::LedoitWolf)]
    pub estimator: Estimator,
}

/// `nalufx allocate`.
#[derive(Debug, Args)]
#[command(
    after_help = "Example:\n  nalufx allocate --assets SPY,TLT,GLD --method risk-parity --max-weight 0.6"
)]
pub struct AllocateArgs {
    /// Where the returns come from.
    #[command(flatten)]
    pub source: ReturnsSource,
    /// The allocation method (`nalufx methods` describes them).
    #[arg(long, value_enum, default_value_t = Method::MinVariance)]
    pub method: Method,
    /// Risk aversion, for mean-variance only.
    #[arg(long, value_parser = positive)]
    pub risk_aversion: Option<f64>,
    /// The bounds and estimator.
    #[command(flatten)]
    pub bounds: Bounds,
    /// Print JSON instead of a table (for scripts and `nalufx report`).
    #[arg(long)]
    pub json: bool,
}

/// `nalufx forecast`.
#[derive(Debug, Args)]
#[command(
    after_help = "Example:\n  nalufx forecast --file nalufx-cli/data/sample-cash.csv --column net_cash_flow --seasonal 5"
)]
pub struct ForecastArgs {
    /// A CSV with the series in a column.
    #[arg(
        long,
        value_name = "PATH",
        required_unless_present = "history",
        conflicts_with = "history"
    )]
    pub file: Option<PathBuf>,
    /// The CSV column to forecast; the last column by default.
    #[arg(long, requires = "file")]
    pub column: Option<String>,
    /// The series itself, comma-separated, oldest first.
    #[arg(long, value_delimiter = ',', value_name = "VALUES")]
    pub history: Option<Vec<f64>>,
    /// Days to forecast.
    #[arg(long, default_value_t = 6, value_parser = clap::value_parser!(u16).range(1..=366))]
    pub days: u16,
    /// A cycle length in the series, such as 5 for a business week; repeat for several.
    #[arg(long, value_name = "PERIOD", value_parser = clap::value_parser!(u16).range(2..))]
    pub seasonal: Vec<u16>,
    /// Print JSON instead of a table.
    #[arg(long)]
    pub json: bool,
}

/// `nalufx backtest`.
#[derive(Debug, Args)]
#[command(
    after_help = "Example:\n  nalufx backtest --file nalufx-cli/data/sample-returns.csv --method all --max-weight 0.4"
)]
pub struct BacktestArgs {
    /// Where the returns come from.
    #[command(flatten)]
    pub source: ReturnsSource,
    /// The method to test, or `all` to compare every method.
    #[arg(long, value_enum, default_value_t = BacktestMethod::All)]
    pub method: BacktestMethod,
    /// Days of history each allocation is estimated from.
    #[arg(long, default_value_t = 120, value_parser = clap::value_parser!(u16).range(2..))]
    pub lookback: u16,
    /// Days between rebalances.
    #[arg(long, default_value_t = 21, value_parser = clap::value_parser!(u16).range(1..))]
    pub rebalance: u16,
    /// The trading cost, in basis points of the value traded.
    #[arg(long, default_value_t = 10.0, value_parser = non_negative)]
    pub cost_bps: f64,
    /// The bounds and estimator.
    #[command(flatten)]
    pub bounds: Bounds,
    /// Print JSON instead of a table.
    #[arg(long)]
    pub json: bool,
}

/// `nalufx report`.
#[derive(Debug, Args)]
#[command(
    after_help = "Example:\n  nalufx allocate --file nalufx-cli/data/sample-returns.csv --json \\\n    | nalufx report --provider anthropic --model claude-sonnet-5-5 --from -"
)]
pub struct ReportArgs {
    /// The provider (`nalufx providers` lists them and their keys); or
    /// `llm.provider` in the configuration file.
    #[arg(long, env = "NALUFX_LLM_PROVIDER")]
    pub provider: Option<String>,
    /// The model's name at the provider; or `llm.model` in the
    /// configuration file. There is no default.
    #[arg(long, env = "NALUFX_LLM_MODEL")]
    pub model: Option<String>,
    /// An allocation as `nalufx allocate --json` prints it; `-` reads standard input.
    #[arg(long, value_name = "PATH")]
    pub from: PathBuf,
    /// The portfolio's name, for the narrative.
    #[arg(long, default_value = "Portfolio")]
    pub portfolio: String,
    /// The investor's objectives, in their words.
    #[arg(long, default_value = "No stated objectives.")]
    pub context: String,
    /// The report's token limit; or `llm.max_tokens` in the
    /// configuration file, or 2048.
    #[arg(long, env = "NALUFX_LLM_MAX_TOKENS")]
    pub max_tokens: Option<u32>,
    /// Print JSON instead of text.
    #[arg(long)]
    pub json: bool,
}

/// An allocation method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Method {
    /// 1/n each: the baseline.
    EqualWeight,
    /// The least variance.
    MinVariance,
    /// Markowitz: return against variance.
    MeanVariance,
    /// Equal risk contribution.
    RiskParity,
    /// Hierarchical risk parity.
    Hrp,
}

impl Method {
    /// Every method, in display order.
    pub const ALL: [Self; 5] =
        [Self::EqualWeight, Self::MinVariance, Self::MeanVariance, Self::RiskParity, Self::Hrp];

    /// The engine's name for it.
    #[must_use]
    pub const fn engine_name(self) -> &'static str {
        match self {
            Self::EqualWeight => "equal_weight",
            Self::MinVariance => "min_variance",
            Self::MeanVariance => "mean_variance",
            Self::RiskParity => "risk_parity",
            Self::Hrp => "hrp",
        }
    }
}

/// A backtest method: one, or all of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum BacktestMethod {
    /// Every method, side by side.
    All,
    /// 1/n each.
    EqualWeight,
    /// The least variance.
    MinVariance,
    /// Markowitz.
    MeanVariance,
    /// Equal risk contribution.
    RiskParity,
    /// Hierarchical risk parity.
    Hrp,
}

impl BacktestMethod {
    /// The methods it stands for.
    #[must_use]
    pub fn methods(self) -> Vec<Method> {
        match self {
            Self::All => Method::ALL.to_vec(),
            Self::EqualWeight => vec![Method::EqualWeight],
            Self::MinVariance => vec![Method::MinVariance],
            Self::MeanVariance => vec![Method::MeanVariance],
            Self::RiskParity => vec![Method::RiskParity],
            Self::Hrp => vec![Method::Hrp],
        }
    }
}

/// A covariance estimator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Estimator {
    /// Ledoit-Wolf shrinkage.
    LedoitWolf,
    /// The sample covariance.
    Sample,
}

/// A number in `[0, 1]`.
fn fraction(s: &str) -> Result<f64, String> {
    let v: f64 = s.parse().map_err(|_| format!("`{s}` is not a number"))?;
    if (0.0..=1.0).contains(&v) {
        Ok(v)
    } else {
        Err(format!("{v} is not between 0 and 1"))
    }
}

/// A finite number above 0.
fn positive(s: &str) -> Result<f64, String> {
    let v: f64 = s.parse().map_err(|_| format!("`{s}` is not a number"))?;
    if v.is_finite() && v > 0.0 {
        Ok(v)
    } else {
        Err(format!("{v} must be above 0"))
    }
}

/// A finite number of at least 0.
fn non_negative(s: &str) -> Result<f64, String> {
    let v: f64 = s.parse().map_err(|_| format!("`{s}` is not a number"))?;
    if v.is_finite() && v >= 0.0 {
        Ok(v)
    } else {
        Err(format!("{v} must not be negative"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn the_command_line_is_well_formed() {
        Cli::command().debug_assert();
    }

    #[test]
    fn number_checks() {
        assert!(fraction("0.4").is_ok() && fraction("1.2").is_err() && fraction("x").is_err());
        assert!(positive("3").is_ok() && positive("0").is_err() && positive("nan").is_err());
        assert!(non_negative("0").is_ok() && non_negative("-1").is_err());
    }

    #[test]
    fn all_expands_to_every_method() {
        assert_eq!(BacktestMethod::All.methods().len(), 5);
        assert_eq!(BacktestMethod::Hrp.methods(), vec![Method::Hrp]);
        assert_eq!(Method::RiskParity.engine_name(), "risk_parity");
    }
}
