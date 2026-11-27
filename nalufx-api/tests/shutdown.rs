//! Graceful shutdown: a request in flight when SIGTERM arrives still
//! completes with 200, and the server then exits cleanly. Its own test
//! binary, because it signals its own process.
#![cfg(unix)]

use async_trait::async_trait;
use nalufx_api::app::{app, AppState};
use nalufx_api::auth::hash_key;
use nalufx_api::config::ApiConfig;
use nalufx_api::server::serve_until_signal;
use nalufx_llms::error::LlmError;
use nalufx_llms::provider::{ChatRequest, ChatResponse, LlmProvider, TokenUsage};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// A provider that takes a second to answer.
#[derive(Debug)]
struct Slow;

#[async_trait]
impl LlmProvider for Slow {
    fn name(&self) -> &'static str {
        "slow"
    }
    fn model(&self) -> &str {
        "m"
    }
    fn base_url(&self) -> &str {
        "http://127.0.0.1:9"
    }
    async fn complete(&self, _: &ChatRequest) -> Result<ChatResponse, LlmError> {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let text =
            r#"{"summary":"s","rationale":["r"],"risks":["k"],"disclaimer":"d"}"#.to_string();
        Ok(ChatResponse {
            text,
            usage: TokenUsage::default(),
            model: "m".into(),
            provider: "slow".into(),
        })
    }
}

/// One HTTP/1.1 request on a raw connection; returns the status line.
async fn post_report(addr: std::net::SocketAddr) -> String {
    let body = r#"{"portfolio":"P","facts":{},"context":"c"}"#;
    let request = format!(
        "POST /v1/report HTTP/1.1\r\nHost: test\r\nAuthorization: Bearer k\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    let mut stream = TcpStream::connect(addr).await.expect("the server accepts");
    stream.write_all(request.as_bytes()).await.expect("the request is sent");
    let mut reply = String::new();
    let _ = stream.read_to_string(&mut reply).await.expect("a reply");
    reply.lines().next().unwrap_or_default().to_string()
}

#[tokio::test]
async fn a_request_in_flight_at_sigterm_completes() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let state = AppState::new(ApiConfig::with_keys(vec![hash_key("k")]), Some(Arc::new(Slow)));
    let server = tokio::spawn(serve_until_signal(listener, app(state)));
    // Let the server install its SIGTERM handler before the signal is sent.
    tokio::time::sleep(Duration::from_millis(200)).await;

    let in_flight = tokio::spawn(post_report(addr));
    tokio::time::sleep(Duration::from_millis(200)).await;
    let status = std::process::Command::new("kill")
        .args(["-TERM", &std::process::id().to_string()])
        .status()
        .unwrap();
    assert!(status.success());

    assert_eq!(in_flight.await.unwrap(), "HTTP/1.1 200 OK", "the in-flight request finished");
    let stopped = tokio::time::timeout(Duration::from_secs(10), server)
        .await
        .expect("the server stopped after draining");
    assert!(stopped.unwrap().is_ok());
    assert!(TcpStream::connect(addr).await.is_err(), "no new connections after shutdown");
}
