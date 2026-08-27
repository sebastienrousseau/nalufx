//! End-to-end use of the Agy adapter.
//!
//! Run against a real deployment:
//!
//! ```sh
//! export AGY_API_KEY=sk-...
//! export AGY_API_URL=https://your-agy-host/v1/chat/completions
//! cargo run -p nalufx-llms --example agy
//! ```
//!
//! With neither variable set the example still runs: it falls back to
//! parsing a canned response, so it demonstrates the shape without
//! needing credentials or a network.

use nalufx_llms::llms::agy::{get_agy_api_key, parse_agy_response, send_agy_request, API_KEY_VAR};
use reqwest::Client;
use serde_json::json;
use std::env;

/// The endpoint is not baked into the adapter, so the example takes it
/// from the environment too.
const URL_VAR: &str = "AGY_API_URL";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let key = match get_agy_api_key() {
        Ok(key) => key,
        Err(err) => {
            println!("No credential ({err}).");
            println!("Set {API_KEY_VAR} and {URL_VAR} to call a real endpoint.\n");
            return offline_demo();
        },
    };

    let Ok(url) = env::var(URL_VAR) else {
        println!(
            "{API_KEY_VAR} is set but {URL_VAR} is not, so there is nowhere to send the request.\n"
        );
        return offline_demo();
    };

    let request = json!({
        "model": "agy-1",
        "messages": [{
            "role": "user",
            "content": "Reply with six space-separated numbers and nothing else.",
        }],
        "max_tokens": 64,
    });

    println!("POST {url}");
    let body = send_agy_request(&Client::new(), &url, &key, request).await?;
    let predictions = parse_agy_response(&body)?;
    println!("parsed {} value(s): {predictions:?}", predictions.len());
    Ok(())
}

/// Demonstrates the parser without a network or a credential.
fn offline_demo() -> Result<(), Box<dyn std::error::Error>> {
    println!("-- offline demonstration --");

    let ok = r#"{"choices":[{"message":{"content":"1.5 2.5 3.0"}}]}"#;
    println!("parsed:        {:?}", parse_agy_response(ok)?);

    // Values are flattened across choices, in order.
    let multi = r#"{"choices":[{"message":{"content":"1.0 2.0"}},
                               {"message":{"content":"3.0"}}]}"#;
    println!("two choices:   {:?}", parse_agy_response(multi)?);

    // A token that is not a number becomes 0.0 rather than failing the
    // batch, so one bad value cannot discard a usable completion.
    let mixed = r#"{"choices":[{"message":{"content":"1.0 oops 3.0"}}]}"#;
    println!("mixed tokens:  {:?}", parse_agy_response(mixed)?);

    // A body of the wrong shape is an error, tagged with the provider.
    match parse_agy_response(r#"{"unexpected":true}"#) {
        Ok(_) => println!("wrong shape:   unexpectedly parsed"),
        Err(err) => println!("wrong shape:   {err}"),
    }

    Ok(())
}
