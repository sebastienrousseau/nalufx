//! # nalufx-api
//!
//! The NaluFX HTTP API on axum and tower: `POST /v1/forecast`,
//! `POST /v1/allocate` and `POST /v1/report` behind bearer API keys and
//! per-key rate limits, with `GET /healthz` and `GET /readyz` open.
//! Every error is RFC 9457 problem details with a stable type URI and the
//! request id, and the contract is OpenAPI 3.1, generated from this code
//! and committed as `openapi.json`. Every request is traced, counted in
//! Prometheus metrics at `GET /metrics`, and allocations can be written
//! to an append-only audit log.

pub mod app;
pub mod auth;
pub mod config;
pub mod metrics;
pub mod openapi;
pub mod problem;
pub mod routes;
pub mod server;
pub mod telemetry;
