//! The committed `openapi.json` is the contract: it must equal the spec
//! the code generates. After a reviewed change to the API, regenerate it
//! with `UPDATE_OPENAPI=1 cargo test -p nalufx-api --test openapi`.

use nalufx_api::openapi::spec_json;
use serde_json::Value;

const PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/openapi.json");

#[test]
fn the_committed_spec_matches_the_code() {
    let generated = spec_json();
    if std::env::var("UPDATE_OPENAPI").is_ok() {
        std::fs::write(PATH, &generated).expect("openapi.json is writable");
    }
    let committed = std::fs::read_to_string(PATH).expect("openapi.json exists");
    assert!(
        committed == generated,
        "openapi.json differs from the code; review the change, then UPDATE_OPENAPI=1"
    );
}

#[test]
fn the_spec_is_openapi_3_1_with_every_route_and_the_key_scheme() {
    let spec: Value = serde_json::from_str(&spec_json()).unwrap();
    assert!(spec["openapi"].as_str().unwrap().starts_with("3.1"));
    for path in
        ["/", "/openapi.json", "/v1/forecast", "/v1/allocate", "/v1/report", "/healthz", "/readyz"]
    {
        assert!(spec["paths"].get(path).is_some(), "{path}");
    }
    assert_eq!(spec["components"]["securitySchemes"]["api_key"]["scheme"], "bearer");
    for open in ["/", "/openapi.json", "/healthz", "/readyz"] {
        assert!(spec["paths"][open]["get"].get("security").is_none(), "{open} is open");
    }
    assert!(spec["paths"]["/v1/allocate"]["post"]["security"].is_array(), "allocation needs a key");
}
