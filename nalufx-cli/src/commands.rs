//! What each command does. Everything is checked before anything slow
//! happens: bounds before prices are fetched, a provider's key before a
//! file is read.

use crate::cli::{
    AllocateArgs, BacktestArgs, Bounds, Cli, Command, Estimator, ForecastArgs, Method, ReportArgs,
};
use crate::error::CliError;
use crate::input;
use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, CellAlignment, Table};
use nalufx_engine::allocate::{by_name, Allocation, AllocationInput, Constraints};
use nalufx_engine::backtest::{run, BacktestConfig, BacktestReport};
use nalufx_engine::covariance::CovarianceEstimator;
use nalufx_engine::forecast::{forecast_seasonal, forecast_time_series, IntervalForecast};
use nalufx_llms::http::Http;
use nalufx_llms::provider::{ClientConfig, LlmConfig};
use nalufx_llms::registry::{from_env, PROVIDERS};
use nalufx_llms::report::{generate, Report, ReportInput};
use std::io::Write;

/// Runs a command, writing its output to `out`.
///
/// # Errors
///
/// A [`CliError`] with its exit code and a hint.
pub async fn execute(cli: Cli, out: &mut impl Write) -> Result<(), CliError> {
    let text = dispatch(cli.command).await?;
    writeln!(out, "{text}").map_err(|e| {
        CliError::input(format!("cannot write the output: {e}"), "check the output pipe")
    })
}

/// The output of one command.
async fn dispatch(command: Command) -> Result<String, CliError> {
    match command {
        Command::Allocate(args) => allocate(&args).await,
        Command::Forecast(args) => forecast(&args),
        Command::Backtest(args) => backtest(&args).await,
        Command::Report(args) => report(&args).await,
        Command::Methods => Ok(methods()),
        Command::Providers => Ok(providers()),
    }
}

/// `nalufx allocate`.
async fn allocate(args: &AllocateArgs) -> Result<String, CliError> {
    let method = by_name(args.method.engine_name(), args.risk_aversion)?;
    check_bounds(&args.bounds, args.source.assets.len())?;
    let (assets, returns) = input::returns(&args.source).await?;
    let constraints = constraints(&args.bounds);
    let input = AllocationInput {
        assets,
        returns,
        constraints,
        estimator: estimator(args.bounds.estimator),
    };
    let allocation = method.allocate(&input)?;
    Ok(if args.json { json(&allocation)? } else { allocation_table(&allocation) })
}

/// `nalufx forecast`.
fn forecast(args: &ForecastArgs) -> Result<String, CliError> {
    let history = history(args)?;
    let days = usize::from(args.days);
    let periods: Vec<usize> = args.seasonal.iter().map(|p| usize::from(*p)).collect();
    let forecast = if periods.is_empty() {
        forecast_time_series(&history, days)?
    } else {
        forecast_seasonal(&history, &periods, days)?
    };
    Ok(if args.json { json(&forecast)? } else { forecast_table(&forecast) })
}

/// The series to forecast, from the file or the command line.
fn history(args: &ForecastArgs) -> Result<Vec<f64>, CliError> {
    match (&args.file, &args.history) {
        (Some(path), _) => input::read_series(path, args.column.as_deref()),
        (None, Some(values)) => Ok(values.clone()),
        (None, None) => {
            Err(CliError::input("no series given", "pass --file PATH or --history 1,2,3,..."))
        },
    }
}

/// `nalufx backtest`.
async fn backtest(args: &BacktestArgs) -> Result<String, CliError> {
    check_bounds(&args.bounds, args.source.assets.len())?;
    let (assets, returns) = input::returns(&args.source).await?;
    let config = BacktestConfig {
        lookback: usize::from(args.lookback),
        rebalance_every: usize::from(args.rebalance),
        cost_bps: args.cost_bps,
        periods_per_year: 252.0,
    };
    let mut reports = Vec::new();
    for method in args.method.methods() {
        let allocator = by_name(method.engine_name(), None)?;
        reports.push(run(
            allocator.as_ref(),
            &assets,
            &returns,
            constraints(&args.bounds),
            estimator(args.bounds.estimator),
            config,
        )?);
    }
    Ok(if args.json { json(&reports)? } else { backtest_table(&reports) })
}

/// `nalufx report`.
async fn report(args: &ReportArgs) -> Result<String, CliError> {
    let config = LlmConfig::new(args.model.clone(), args.max_tokens)?;
    let http = Http::new(ClientConfig::default())?;
    let provider = from_env(&args.provider, config, http)?;
    let text = input::read_text(&args.from)?;
    let allocation: Allocation = serde_json::from_str(&text).map_err(|e| {
        CliError::input(
            format!("{}: not an allocation: {e}", args.from.display()),
            "pass the output of `nalufx allocate --json`",
        )
    })?;
    let input = ReportInput {
        portfolio: args.portfolio.clone(),
        facts: serde_json::to_value(&allocation).unwrap_or_default(),
        context: args.context.clone(),
    };
    let report = generate(provider.as_ref(), &input).await?;
    Ok(if args.json { json(&report)? } else { report_text(&report) })
}

/// `nalufx methods`.
fn methods() -> String {
    let mut t = table(&["Method", "What it does"]);
    let rows = [
        (Method::EqualWeight, "1/n to every asset; the baseline every other method must beat"),
        (
            Method::MinVariance,
            "the least-variance portfolio; ignores expected returns (the default)",
        ),
        (Method::MeanVariance, "Markowitz: return against variance, --risk-aversion (default 10)"),
        (Method::RiskParity, "each asset contributes the same share of the variance"),
        (Method::Hrp, "hierarchical risk parity; stable when assets are highly correlated"),
    ];
    for (m, what) in rows {
        let _ = t.add_row(vec![Cell::new(kebab(m.engine_name())), Cell::new(what)]);
    }
    t.to_string()
}

/// `nalufx providers`.
fn providers() -> String {
    let mut t = table(&["Provider", "Key variable", "Key set"]);
    for p in PROVIDERS {
        let var = key_variable(p);
        let set = var.map_or("not needed", |v| {
            if std::env::var(v).is_ok_and(|k| !k.is_empty()) {
                "yes"
            } else {
                "no"
            }
        });
        let _ = t.add_row(vec![p, var.unwrap_or("-"), set]);
    }
    format!("{t}\nAgy and openai_compatible also need NALUFX_LLM_BASE_URL.")
}

/// The environment variable that holds a provider's key, if it needs one.
fn key_variable(provider: &str) -> Option<&'static str> {
    match provider {
        "anthropic" => Some("ANTHROPIC_API_KEY"),
        "gemini" => Some("GEMINI_API_KEY"),
        "openai" => Some("OPENAI_API_KEY"),
        "mistral" => Some("MISTRAL_API_KEY"),
        "agy" => Some("AGY_API_KEY"),
        _ => None,
    }
}

/// Rejects bounds no weights can meet, before any price is fetched. With a
/// file the asset count is only known after reading it, so the engine
/// checks again then.
fn check_bounds(bounds: &Bounds, tickers: usize) -> Result<(), CliError> {
    let c = constraints(bounds);
    if bounds.min_weight > bounds.max_weight {
        return Err(CliError::input(
            "--min-weight is above --max-weight",
            "lower --min-weight or raise --max-weight",
        ));
    }
    if tickers > 0 {
        c.check(tickers)?;
    }
    Ok(())
}

fn constraints(b: &Bounds) -> Constraints {
    Constraints { max_weight: b.max_weight, min_weight: b.min_weight }
}

const fn estimator(e: Estimator) -> CovarianceEstimator {
    match e {
        Estimator::LedoitWolf => CovarianceEstimator::LedoitWolf,
        Estimator::Sample => CovarianceEstimator::Sample,
    }
}

/// Pretty JSON.
fn json(value: &impl serde::Serialize) -> Result<String, CliError> {
    serde_json::to_string_pretty(value)
        .map_err(|e| CliError::input(format!("cannot write JSON: {e}"), "report this as a bug"))
}

/// A table with a header.
fn table(header: &[&str]) -> Table {
    let mut t = Table::new();
    let _ = t.load_style(UTF8_FULL_CONDENSED).set_header(header.to_vec());
    t
}

/// `min_variance` as `min-variance`, as the command line spells it.
fn kebab(name: &str) -> String {
    name.replace('_', "-")
}

/// An allocation as a table and a line of diagnostics.
fn allocation_table(a: &Allocation) -> String {
    let mut t = table(&["Asset", "Weight"]);
    for (asset, w) in a.assets.iter().zip(&a.weights) {
        let _ = t.add_row(vec![
            Cell::new(asset),
            Cell::new(format!("{:.2}%", w * 100.0)).set_alignment(CellAlignment::Right),
        ]);
    }
    let d = &a.diagnostics;
    let shrinkage = d.shrinkage.map_or_else(|| "none".to_string(), |s| format!("{s:.3}"));
    let projected = if d.projected { "; projected onto the bounds" } else { "" };
    format!(
        "{} allocation\n{t}\nsolver {} in {} iterations, covariance condition number {:.1}, shrinkage {shrinkage}{projected}",
        kebab(&a.method), d.solver_status, d.iterations, d.condition_number
    )
}

/// A forecast as a table, one row per day.
fn forecast_table(f: &IntervalForecast) -> String {
    let mut t = table(&["Day", "Forecast", "Lower 95%", "Upper 95%"]);
    for day in 0..f.point.len() {
        let n = |v: f64| Cell::new(format!("{v:.2}")).set_alignment(CellAlignment::Right);
        let _ =
            t.add_row(vec![Cell::new(day + 1), n(f.point[day]), n(f.lower[day]), n(f.upper[day])]);
    }
    t.to_string()
}

/// Backtest reports side by side.
fn backtest_table(reports: &[BacktestReport]) -> String {
    let mut t =
        table(&["Method", "Return/yr", "Volatility/yr", "Sharpe", "Max drawdown", "Turnover/yr"]);
    for r in reports {
        let pct =
            |v: f64| Cell::new(format!("{:.2}%", v * 100.0)).set_alignment(CellAlignment::Right);
        let num = |v: f64| Cell::new(format!("{v:.2}")).set_alignment(CellAlignment::Right);
        let _ = t.add_row(vec![
            Cell::new(kebab(&r.method)),
            pct(r.annualised_return),
            pct(r.annualised_volatility),
            num(r.sharpe),
            pct(r.max_drawdown),
            num(r.annual_turnover),
        ]);
    }
    let c = reports.first().map(|r| r.config);
    let summary = c.map_or_else(String::new, |c| {
        format!("\n{} days after a {}-day lookback, rebalancing every {} days, {} bps per trade; past results only.", reports[0].periods, c.lookback, c.rebalance_every, c.cost_bps)
    });
    format!("{t}{summary}")
}

/// A report as plain text.
fn report_text(r: &Report) -> String {
    let rationale: String = r.report.rationale.iter().map(|p| format!("- {p}\n")).collect();
    let risks: String = r.report.risks.iter().map(|p| format!("- {p}\n")).collect();
    format!(
        "{}\n\nWhy:\n{rationale}\nRisks:\n{risks}\n{}\n\n(written by {} {}, prompt {} v{})",
        r.report.summary, r.report.disclaimer, r.provider, r.model, r.prompt.name, r.prompt.version
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_method_and_provider_is_listed() {
        let m = methods();
        for name in ["equal-weight", "min-variance", "mean-variance", "risk-parity", "hrp"] {
            assert!(m.contains(name), "{name}");
        }
        let p = providers();
        for name in PROVIDERS {
            assert!(p.contains(name), "{name}");
        }
    }

    #[test]
    fn bounds_no_weights_can_meet_are_caught_before_fetching() {
        let b = Bounds { max_weight: 0.2, min_weight: 0.0, estimator: Estimator::LedoitWolf };
        assert_eq!(check_bounds(&b, 3).unwrap_err().exit_code(), 4);
        assert!(check_bounds(&b, 0).is_ok(), "with a file the count is not known yet");
        let crossed = Bounds { max_weight: 0.2, min_weight: 0.3, estimator: Estimator::Sample };
        assert_eq!(check_bounds(&crossed, 0).unwrap_err().exit_code(), 3);
    }
}
