//! What each `nalufx tui` screen shows.

use super::job::{Job, Outcome};
use super::keys::ScreenKeys;
use super::model::{App, Screen};
use crate::commands::{kebab, solver_line};
use crate::error::CliError;
use petitcarnets::bubbles::table::{Column, Row};
use petitcarnets::style::{wrap, Color, Style};

const TITLE: Style = Style::new().bold();
const ERROR: Style = Style::new().fg(Color::Palette(9));
const FAINT: Style = Style::new().fg(Color::Palette(241));

/// The whole screen.
pub(crate) fn render(app: &App) -> String {
    let body = match app.screen {
        Screen::Tickers => app.tickers.view(),
        Screen::Methods => app.methods.view(),
        Screen::Days => format!("{}\n\n{}", heading(app), app.days.view()),
        Screen::Running => format!("{} {}", app.spinner.view(), running(app)),
        Screen::Results => results(app),
    };
    let help = match app.screen {
        Screen::Tickers | Screen::Methods => String::new(),
        screen => {
            format!("\n\n{}", app.help.view(&ScreenKeys { screen, table: &app.table.key_map }))
        },
    };
    let width = usize::from(app.size.width);
    let error = app.error.as_ref().map_or_else(String::new, |e| error_text(e, width));
    format!("{body}{error}{help}")
}

/// What the days are for.
fn heading(app: &App) -> String {
    TITLE.render(&format!("Forecast {}", app_tickers(app)))
}

/// The chosen tickers, comma-separated.
fn app_tickers(app: &App) -> String {
    let chosen: Vec<&str> =
        app.tickers.items().iter().filter(|t| t.chosen).map(|t| t.symbol.as_str()).collect();
    chosen.join(", ")
}

/// What the spinner is waiting for.
fn running(app: &App) -> String {
    match app.job {
        Some(Job::Allocate(m)) => {
            format!("Allocating {} over {}...", kebab(m.engine_name()), app_tickers(app))
        },
        Some(Job::Forecast { days }) => {
            format!("Forecasting {days} days of {}...", app_tickers(app))
        },
        None => String::new(),
    }
}

/// The results table, its title, and the solver's line for weights.
fn results(app: &App) -> String {
    let (title, footer) = match &app.outcome {
        Some(Outcome::Weights(a)) => (format!("{} allocation", kebab(&a.method)), solver_line(a)),
        Some(Outcome::Forecasts(f)) => {
            let days = f.first().map_or(0, |(_, x)| x.point.len());
            (format!("{days}-day forecast, 95% interval"), String::new())
        },
        None => (String::new(), String::new()),
    };
    format!("{}\n\n{}\n{}", TITLE.render(&title), app.table.view(), FAINT.render(&footer))
}

/// An error as `nalufx` prints it, with its hint, wrapped at `columns`
/// after a blank line.
pub(crate) fn error_text(e: &CliError, columns: usize) -> String {
    let hint = e.hint().map_or_else(String::new, |h| format!("\n  hint: {h}"));
    format!("\n\n{}", ERROR.render(&wrap(&format!("error: {e}{hint}"), columns)))
}

/// The table's columns and rows for an outcome.
pub(crate) fn table_of(outcome: &Outcome) -> (Vec<Column>, Vec<Row>) {
    match outcome {
        Outcome::Weights(a) => {
            let rows = a
                .assets
                .iter()
                .zip(&a.weights)
                .map(|(asset, w)| vec![asset.clone(), format!("{:>7.2}%", w * 100.0)])
                .collect();
            (vec![Column::new("Asset", 12), Column::new("Weight", 8)], rows)
        },
        Outcome::Forecasts(forecasts) => {
            let columns = ["Ticker", "Day", "Forecast", "Lower 95%", "Upper 95%"]
                .iter()
                .zip([10, 4, 11, 11, 11])
                .map(|(t, w)| Column::new(t, w))
                .collect();
            (columns, forecasts.iter().flat_map(|(t, f)| forecast_rows(t, f)).collect())
        },
    }
}

/// One row per day of a ticker's forecast.
fn forecast_rows(ticker: &str, f: &nalufx_engine::forecast::IntervalForecast) -> Vec<Row> {
    let n = |v: f64| format!("{v:>11.6}");
    (0..f.point.len())
        .map(|d| {
            vec![
                ticker.to_string(),
                (d + 1).to_string(),
                n(f.point[d]),
                n(f.lower[d]),
                n(f.upper[d]),
            ]
        })
        .collect()
}
