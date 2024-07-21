#![doc(
    html_favicon_url = "https://kura.pro/nalufx/images/favicon.ico",
    html_logo_url = "https://kura.pro/nalufx/images/logos/nalufx.svg",
    html_root_url = "https://docs.rs/nalufx"
)]
#![crate_name = "nalufx_llms"]
#![crate_type = "lib"]

extern crate actix_web;
extern crate dotenvy;
extern crate log;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

/// This module contains the logic for interacting with the OpenAI API.
pub mod llms;

/// This module contains the data models for the OpenAI API.
pub mod models;
