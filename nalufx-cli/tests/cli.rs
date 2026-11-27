//! The `nalufx` binary, end to end: real arguments, real files, real exit
//! codes. Nothing here reaches the network; `report` talks to a local mock.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn data(file: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data").join(file)
}

/// Runs `nalufx` in an empty directory (so no `.env` is picked up) with no
/// provider keys set.
fn nalufx(args: &[&str]) -> Output {
    let dir = tempfile::tempdir().unwrap();
    Command::new(env!("CARGO_BIN_EXE_nalufx"))
        .args(args)
        .current_dir(dir.path())
        .env_remove("ANTHROPIC_API_KEY")
        .env_remove("NALUFX_LLM_PROVIDER")
        .env_remove("NALUFX_LLM_MODEL")
        .output()
        .expect("nalufx runs")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

#[test]
fn no_arguments_prints_help_and_exits_2() {
    let o = nalufx(&[]);
    assert_eq!(o.status.code(), Some(2));
    assert!(stderr(&o).contains("allocate"), "{}", stderr(&o));
}

#[test]
fn allocate_from_a_file_prints_a_table_or_json() {
    let file = data("sample-returns.csv");
    let file = file.to_str().unwrap();
    let o = nalufx(&["allocate", "--file", file, "--method", "risk-parity"]);
    assert!(o.status.success(), "{}", stderr(&o));
    for asset in ["BOND", "CREDIT", "EQUITY", "SMALL", "GOLD"] {
        assert!(stdout(&o).contains(asset), "{asset}");
    }
    let o =
        nalufx(&["allocate", "--file", file, "--method", "hrp", "--max-weight", "0.4", "--json"]);
    let a: serde_json::Value = serde_json::from_slice(&o.stdout).expect("JSON");
    assert_eq!(a["method"], "hrp");
    let total: f64 = a["weights"].as_array().unwrap().iter().map(|w| w.as_f64().unwrap()).sum();
    assert!((total - 1.0).abs() < 1e-9);
    assert!(a["weights"].as_array().unwrap().iter().all(|w| w.as_f64().unwrap() <= 0.4 + 1e-12));
}

#[test]
fn backtest_all_compares_every_method() {
    let file = data("sample-returns.csv");
    let o = nalufx(&["backtest", "--file", file.to_str().unwrap(), "--json"]);
    assert!(o.status.success(), "{}", stderr(&o));
    let reports: Vec<serde_json::Value> = serde_json::from_slice(&o.stdout).expect("JSON");
    let methods: Vec<&str> = reports.iter().map(|r| r["method"].as_str().unwrap()).collect();
    assert_eq!(methods, ["equal_weight", "min_variance", "mean_variance", "risk_parity", "hrp"]);
}

#[test]
fn forecast_from_a_file_column_or_inline_values() {
    let file = data("sample-cash.csv");
    let o = nalufx(&[
        "forecast",
        "--file",
        file.to_str().unwrap(),
        "--column",
        "net_cash_flow",
        "--seasonal",
        "5",
        "--days",
        "5",
    ]);
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(stdout(&o).contains("Upper 95%"));
    let o = nalufx(&[
        "forecast",
        "--history",
        "100,101,103,102,104,105,107,106,108,110",
        "--days",
        "3",
        "--json",
    ]);
    let f: serde_json::Value = serde_json::from_slice(&o.stdout).expect("JSON");
    assert_eq!(f["point"].as_array().unwrap().len(), 3);
}

#[test]
fn methods_and_providers_list_the_choices() {
    let o = nalufx(&["methods"]);
    assert!(o.status.success() && stdout(&o).contains("risk-parity"));
    let o = nalufx(&["providers"]);
    assert!(o.status.success() && stdout(&o).contains("ANTHROPIC_API_KEY"));
}

#[test]
fn a_bad_value_is_a_usage_error_with_the_choices() {
    let o = nalufx(&["allocate", "--file", "x.csv", "--method", "magic"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(stderr(&o).contains("possible values"), "{}", stderr(&o));
    assert_eq!(
        nalufx(&["allocate", "--file", "x.csv", "--max-weight", "1.5"]).status.code(),
        Some(2)
    );
    assert_eq!(nalufx(&["allocate"]).status.code(), Some(2), "a source is required");
}

#[test]
fn impossible_bounds_fail_before_any_price_is_fetched() {
    let o = nalufx(&["allocate", "--assets", "SPY,TLT,GLD", "--max-weight", "0.2"]);
    assert_eq!(o.status.code(), Some(4));
    assert!(stderr(&o).contains("hint: raise --max-weight"), "{}", stderr(&o));
}

#[test]
fn unreadable_data_is_an_input_error_naming_the_problem() {
    let o = nalufx(&["allocate", "--file", "/no/such/file.csv"]);
    assert_eq!(o.status.code(), Some(3));
    let dir = tempfile::tempdir().unwrap();
    let bad = dir.path().join("bad.csv");
    std::fs::write(&bad, "A,B\n0.01,0.02\n0.03,oops\n").unwrap();
    let o = nalufx(&["allocate", "--file", bad.to_str().unwrap()]);
    assert_eq!(o.status.code(), Some(3));
    assert!(stderr(&o).contains("row 3, column `B`"), "{}", stderr(&o));
}

#[test]
fn a_missing_key_fails_before_reading_the_input() {
    let o = nalufx(&[
        "report",
        "--provider",
        "anthropic",
        "--model",
        "m",
        "--from",
        "/no/such/allocation.json",
    ]);
    assert_eq!(o.status.code(), Some(3));
    assert!(stderr(&o).contains("ANTHROPIC_API_KEY"), "{}", stderr(&o));
}

#[tokio::test]
async fn allocate_json_pipes_into_a_report() {
    let server = MockServer::start().await;
    let report = r#"{"summary":"Bonds lead.","rationale":["Lowest volatility."],"risks":["Rates."],"disclaimer":"Research only."}"#;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({ "choices": [{ "message": { "content": report } }], "model": "stub" })))
        .mount(&server)
        .await;
    let dir = tempfile::tempdir().unwrap();
    let allocation = dir.path().join("allocation.json");
    let file = data("sample-returns.csv");
    let o = nalufx(&["allocate", "--file", file.to_str().unwrap(), "--json"]);
    std::fs::write(&allocation, &o.stdout).unwrap();

    let url = format!("{}/v1", server.uri());
    let alloc = allocation.to_str().unwrap().to_string();
    let o = tokio::task::spawn_blocking(move || {
        Command::new(env!("CARGO_BIN_EXE_nalufx"))
            .args([
                "report",
                "--provider",
                "openai_compatible",
                "--model",
                "stub",
                "--from",
                &alloc,
            ])
            .env("NALUFX_LLM_BASE_URL", url)
            .output()
            .unwrap()
    })
    .await
    .unwrap();
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(stdout(&o).contains("Bonds lead."));
    assert!(stdout(&o).contains("prompt allocation_report v1"));
}
