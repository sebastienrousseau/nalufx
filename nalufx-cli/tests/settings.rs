//! `nalufx serve` and `nalufx config check`, end to end, as a process.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

const HASH: &str = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";

/// `nalufx` in `dir`, with no server or provider settings from the
/// caller's environment.
fn nalufx(dir: &Path) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_nalufx"));
    let _ = c.current_dir(dir);
    for (name, _) in std::env::vars() {
        if name.starts_with("NALUFX_") || name.ends_with("_API_KEY") || name.starts_with("OTEL_") {
            let _ = c.env_remove(name);
        }
    }
    c
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn check(dir: &Path, toml: &str, args: &[&str]) -> Output {
    std::fs::write(dir.join("nalufx.toml"), toml).unwrap();
    nalufx(dir).args(["config", "check", "--config", "nalufx.toml"]).args(args).output().unwrap()
}

#[test]
fn config_check_shows_each_setting_and_its_source() {
    let dir = tempfile::tempdir().unwrap();
    let toml = format!("[server]\napi_key_sha256 = [\"{HASH}\"]\naddr = \"127.0.0.1:9100\"\n");
    std::fs::write(dir.path().join("nalufx.toml"), toml).unwrap();
    let o = nalufx(dir.path())
        .args(["config", "check", "--config", "nalufx.toml", "--json"])
        .env("NALUFX_CONCURRENCY", "8")
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0), "{}", text(&o.stderr));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    let setting = |key: &str| {
        v["settings"].as_array().unwrap().iter().find(|s| s["key"] == key).unwrap().clone()
    };
    assert_eq!(setting("server.addr")["value"], "127.0.0.1:9100");
    assert_eq!(setting("server.addr")["source"], "file");
    assert_eq!(setting("limits.concurrency")["source"], "environment");
    assert_eq!(setting("limits.rate_per_minute")["source"], "default");
    assert_eq!(setting("server.api_key_sha256")["value"], "1 key(s), ids 9f86d081");
    assert!(!text(&o.stdout).contains(HASH), "a key hash was printed");
}

#[test]
fn config_check_names_the_bad_key_and_exits_3() {
    let dir = tempfile::tempdir().unwrap();
    let o =
        check(dir.path(), &format!("[server]\napi_key_sha256 = [\"{HASH}\"]\naddr = \"x\"\n"), &[]);
    assert_eq!(o.status.code(), Some(3));
    assert!(text(&o.stderr).contains("server.addr in nalufx.toml"), "{}", text(&o.stderr));
    let o = check(dir.path(), "[server]\nport = 1\n", &[]);
    assert_eq!(o.status.code(), Some(3));
    assert!(text(&o.stderr).contains("unknown setting `server.port`"), "{}", text(&o.stderr));
}

#[test]
fn config_check_fails_when_the_provider_cannot_start() {
    let dir = tempfile::tempdir().unwrap();
    let toml = format!(
        "[server]\napi_key_sha256 = [\"{HASH}\"]\n[llm]\nprovider = \"anthropic\"\nmodel = \"m\"\n"
    );
    let o = check(dir.path(), &toml, &[]);
    assert_eq!(o.status.code(), Some(3));
    assert!(text(&o.stderr).contains("ANTHROPIC_API_KEY"), "{}", text(&o.stderr));
}

/// A port nothing listens on, a moment ago.
fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port()
}

/// `GET /healthz`'s status line, if the server answers.
fn healthz(port: u16) -> Option<String> {
    let mut s = TcpStream::connect(("127.0.0.1", port)).ok()?;
    s.write_all(b"GET /healthz HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n").ok()?;
    let mut reply = String::new();
    let _ = s.read_to_string(&mut reply).ok()?;
    reply.lines().next().map(str::to_string)
}

/// Regression: `nalufx` held standard output's lock for the whole
/// command, so `serve`'s logger thread blocked on its first line, and the
/// server neither logged nor stopped on SIGTERM.
#[cfg(unix)]
#[test]
fn serve_logs_answers_and_stops_on_sigterm() {
    let dir = tempfile::tempdir().unwrap();
    let port = free_port();
    std::fs::write(
        dir.path().join("nalufx.toml"),
        format!("[server]\napi_key_sha256 = [\"{HASH}\"]\n"),
    )
    .unwrap();
    let mut child = nalufx(dir.path())
        .args(["serve", "--config", "nalufx.toml", "--addr", &format!("127.0.0.1:{port}")])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let start = Instant::now();
    let status = loop {
        if let Some(line) = healthz(port) {
            break line;
        }
        assert!(start.elapsed() < Duration::from_secs(20), "the server never answered");
        std::thread::sleep(Duration::from_millis(50));
    };
    assert!(status.contains("200"), "{status}");

    let killed = Command::new("kill").args(["-TERM", &child.id().to_string()]).status().unwrap();
    assert!(killed.success());
    let start = Instant::now();
    let code = loop {
        if let Some(code) = child.try_wait().unwrap() {
            break code;
        }
        if start.elapsed() > Duration::from_secs(15) {
            child.kill().unwrap();
            panic!("the server did not stop within 15 s of SIGTERM");
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    assert!(code.success(), "{code}");
    let mut logs = String::new();
    let _ = child.stdout.take().unwrap().read_to_string(&mut logs).unwrap();
    assert!(logs.contains("listening on"), "no start-up line in: {logs}");
    assert!(logs.contains("shutting down"), "no shutdown line in: {logs}");
}

/// `nalufx report` takes the provider and model from the file, and a
/// blank `NALUFX_LLM_PROVIDER` (as CI sets for a missing secret) does not
/// hide the file's value, as it does not for the server.
#[test]
fn report_reads_the_file_under_a_blank_variable() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("nalufx.toml"),
        "[llm]\nprovider = \"from-the-file\"\nmodel = \"m\"\n",
    )
    .unwrap();
    let o = nalufx(dir.path())
        .args(["report", "--config", "nalufx.toml", "--from", "-"])
        .env("NALUFX_LLM_PROVIDER", "")
        .env("NALUFX_LLM_MODEL", " ")
        .output()
        .unwrap();
    let err = text(&o.stderr);
    assert_eq!(o.status.code(), Some(3), "{err}");
    assert!(err.contains("from-the-file"), "the file's provider was not used: {err}");
}
