//! # nalufx-llms
//!
//! One [`LlmProvider`](provider::LlmProvider) trait with typed requests and
//! replies, three adapters, and schema-checked narrative reports.
//!
//! - [`adapters::anthropic`]: Anthropic's Messages API.
//! - [`adapters::gemini`]: Google's Gemini `generateContent` API.
//! - [`adapters::openai_compatible`]: everything that speaks OpenAI's chat
//!   completions: OpenAI, Mistral, Ollama, Agy and Llama servers.
//!
//! Every provider shares one HTTP client ([`http::Http`]) with connect and
//! request timeouts, and retries a timeout, a transport failure, `429` or
//! a `5xx` with exponential backoff and jitter, honouring `Retry-After`.
//! The model is always the caller's choice: the library has no default.
//!
//! Language models write words only. [`report::generate`] hands one the
//! computed figures and a JSON Schema, and rejects any reply that does not
//! match it.
#![doc(
    html_favicon_url = "https://kura.pro/nalufx/images/favicon.ico",
    html_logo_url = "https://kura.pro/nalufx/images/logos/nalufx.svg",
    html_root_url = "https://docs.rs/nalufx"
)]
#![crate_name = "nalufx_llms"]
#![crate_type = "lib"]

pub mod adapters;
pub mod error;
pub mod http;
pub mod prompt;
pub mod provider;
pub mod registry;
pub mod report;
pub mod secret;
