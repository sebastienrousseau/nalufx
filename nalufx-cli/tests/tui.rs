//! `nalufx tui` in petitcarnets' test harness: the real loop on a virtual
//! clock and an emulated terminal, with the sample returns file, so
//! every screen is the same on every run and nothing reaches the
//! network. Regenerate the golden files with
//! `PETITCARNETS_UPDATE_GOLDEN=1 cargo test -p nalufx-cli --test tui`.

use nalufx_cli::cli::{Bounds, Estimator, ReturnsSource};
use nalufx_cli::tui::{App, Outcome, Screen, Source};
use petitcarnets as tea;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;
use tea::testing::{assert_golden, TestProgram, TestTerminal};
use tea::{Key, KeyCode};

const WITHIN: Duration = Duration::from_secs(5);

fn sample_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data/sample-returns.csv")
}

fn bounds(max_weight: f64) -> Bounds {
    Bounds { max_weight, min_weight: 0.0, estimator: Estimator::LedoitWolf }
}

fn app(max_weight: f64) -> App {
    let source = ReturnsSource { assets: Vec::new(), file: Some(sample_path()) };
    App::new(Source::from_args(&source).unwrap(), bounds(max_weight), None)
}

fn start() -> TestProgram<App> {
    TestProgram::new(app(1.0), 80, 24)
}

fn press(test: &mut TestProgram<App>, code: KeyCode, times: usize) {
    for _ in 0..times {
        test.press(Key::new(code));
    }
}

/// The method list's rows: the five methods, then forecast.
fn choose(test: &mut TestProgram<App>, row: usize) {
    press(test, KeyCode::Enter, 1);
    assert_eq!(test.model().screen(), Screen::Methods);
    press(test, KeyCode::Down, row);
    press(test, KeyCode::Enter, 1);
}

/// Waits until the screen shows `text`. A task's result is handled when
/// `press` returns, but its frame waits for the next frame-rate tick,
/// which comes only when the virtual clock moves.
fn wait_text(test: &mut TestProgram<App>, text: &str) {
    assert!(
        test.wait_for(WITHIN, |t| t.screen().text().contains(text)),
        "no {text:?} in:\n{}",
        test.screen().text()
    );
}

fn weights(test: &TestProgram<App>) -> Vec<(String, f64)> {
    match test.model().outcome() {
        Some(Outcome::Weights(a)) => a.assets.iter().cloned().zip(a.weights.clone()).collect(),
        other => panic!("no weights: {other:?}"),
    }
}

#[test]
fn the_ticker_list_starts_with_every_ticker_chosen() {
    let test = start();
    assert_golden("tui-tickers", &test.screen().snapshot());
}

#[test]
fn min_variance_matches_nalufx_allocate() {
    let mut test = start();
    choose(&mut test, 1);
    assert_eq!(test.model().screen(), Screen::Results, "{}", test.screen().text());
    wait_text(&mut test, "min-variance allocation");
    assert_golden("tui-min-variance", &test.screen().snapshot());

    let out = Command::new(env!("CARGO_BIN_EXE_nalufx"))
        .args(["allocate", "--file"])
        .arg(sample_path())
        .args(["--method", "min-variance", "--json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let cli: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let cli_weights: Vec<f64> =
        cli["weights"].as_array().unwrap().iter().map(|w| w.as_f64().unwrap()).collect();
    let tui: Vec<f64> = weights(&test).into_iter().map(|(_, w)| w).collect();
    assert_eq!(tui, cli_weights, "the interface and the command disagree");

    // The solver's line, word for word as the command prints it.
    let out = Command::new(env!("CARGO_BIN_EXE_nalufx"))
        .args(["allocate", "--file"])
        .arg(sample_path())
        .args(["--method", "min-variance"])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    let line = text.lines().find(|l| l.starts_with("solver ")).expect("a solver line");
    assert!(test.screen().text().contains(line), "no {line:?} in:\n{}", test.screen().text());
}

#[test]
fn space_leaves_a_ticker_out() {
    let mut test = start();
    test.type_text(" ");
    choose(&mut test, 0);
    let w = weights(&test);
    assert_eq!(
        w.iter().map(|(a, _)| a.as_str()).collect::<Vec<_>>(),
        ["CREDIT", "EQUITY", "SMALL", "GOLD"]
    );
    assert!(w.iter().all(|(_, x)| (x - 0.25).abs() < 1e-12), "{w:?}");
}

#[test]
fn forecast_asks_for_days_then_shows_each_ticker() {
    let mut test = start();
    choose(&mut test, 5);
    assert_eq!(test.model().screen(), Screen::Days);
    assert_golden("tui-days", &test.screen().snapshot());
    press(&mut test, KeyCode::Backspace, 1);
    test.type_text("3");
    press(&mut test, KeyCode::Enter, 1);
    let Some(Outcome::Forecasts(f)) = test.model().outcome() else {
        panic!("no forecast: {}", test.screen().text());
    };
    assert_eq!(f.len(), 5);
    assert!(f.iter().all(|(_, x)| x.point.len() == 3));
    wait_text(&mut test, "3-day forecast");
    assert_golden("tui-forecast", &test.screen().snapshot());
}

#[test]
fn bad_days_show_an_error_and_keep_the_input() {
    let mut test = start();
    choose(&mut test, 5);
    press(&mut test, KeyCode::Backspace, 1);
    test.type_text("0");
    press(&mut test, KeyCode::Enter, 1);
    assert_eq!(test.model().screen(), Screen::Days);
    assert!(test.screen().text().contains("error: `0` is not a number of days"));
}

#[test]
fn an_engine_error_shows_in_the_view_not_as_a_panic() {
    // Five tickers cannot sum to 1 with no weight above 10%.
    let mut test = TestProgram::new(app(0.1), 80, 24);
    choose(&mut test, 1);
    assert_eq!(test.model().screen(), Screen::Methods);
    // The error `nalufx allocate` exits with, word for word.
    let out = Command::new(env!("CARGO_BIN_EXE_nalufx"))
        .args(["allocate", "--file"])
        .arg(sample_path())
        .args(["--method", "min-variance", "--max-weight", "0.1"])
        .output()
        .unwrap();
    let error = test.model().error().expect("an error");
    assert_eq!(out.status.code(), Some(i32::from(error.exit_code())));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains(&format!("error: {error}")), "{stderr}");
    wait_text(&mut test, "error: ");
    assert_golden("tui-error", &test.screen().snapshot());
    // The next key clears it, and the program goes on.
    press(&mut test, KeyCode::Esc, 1);
    assert_eq!(test.model().screen(), Screen::Tickers);
    assert!(test.model().error().is_none());
}

#[test]
fn no_ticker_chosen_is_an_error() {
    let mut test = start();
    for _ in 0..5 {
        test.type_text(" ");
        press(&mut test, KeyCode::Down, 1);
    }
    press(&mut test, KeyCode::Enter, 1);
    assert_eq!(test.model().screen(), Screen::Tickers);
    assert!(test.screen().text().contains("error: no ticker chosen"), "{}", test.screen().text());
}

#[test]
fn q_types_into_the_filter_and_quits_outside_it() {
    let mut test = start();
    test.type_text("/q");
    assert!(!test.wait_finished(Duration::from_millis(10)), "q in the filter quit");
    press(&mut test, KeyCode::Esc, 1);
    test.type_text("q");
    assert!(test.wait_finished(WITHIN));
}

#[test]
fn ctrl_c_quits_from_the_days_input_and_restores_the_terminal() {
    let mut test = start();
    choose(&mut test, 5);
    test.press(Key::ctrl('c'));
    assert!(test.wait_finished(WITHIN));
    let screen = test.final_screen();
    assert!(!screen.alt_screen(), "still on the alternate screen");
    assert!(screen.cursor_visible(), "the cursor is still hidden");
}

#[test]
fn accessible_mode_prints_plain_lines() {
    let program = tea::Program::new(app(1.0)).with_accessible(true);
    let mut test = TestProgram::with_terminal(program, TestTerminal::new(80, 24));
    choose(&mut test, 0);
    test.type_text("q");
    assert!(test.wait_finished(WITHIN));
    let out = String::from_utf8_lossy(test.final_output()).into_owned();
    // The keyboard and paste modes are set and unset; nothing else is
    // written but text: no colour, cursor movement or alternate screen.
    let text =
        ["\u{1b}[?2004h", "\u{1b}[>4;2m", "\u{1b}[>1u", "\u{1b}[>4m", "\u{1b}[<u", "\u{1b}[?2004l"]
            .iter()
            .fold(out.clone(), |o, mode| o.replace(mode, ""));
    assert!(!text.contains('\u{1b}'), "escape sequences in accessible output: {text:?}");
    assert!(out.contains("equal-weight allocation"), "{out}");
    assert!(out.contains("20.00%"), "{out}");
}
