//! The OpenAPI 3.1 contract, generated from the code.
//!
//! The committed `nalufx-api/openapi.json` must equal [`spec_json`]; a test
//! fails when they differ, and `UPDATE_OPENAPI=1` rewrites the file after
//! a reviewed change.

use crate::problem::Problem;
use crate::routes::{self, AllocateRequest, ForecastRequest, Health, Index};
use nalufx_engine::allocate::{Allocation, Constraints, Diagnostics};
use nalufx_engine::covariance::CovarianceEstimator;
use nalufx_engine::forecast::IntervalForecast;
use nalufx_llms::prompt::PromptId;
use nalufx_llms::provider::TokenUsage;
use nalufx_llms::report::{AllocationReport, Report, ReportInput};
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

/// The API's OpenAPI document.
#[derive(Clone, Copy, Debug, OpenApi)]
#[openapi(
    info(title = "NaluFX API", description = "Deterministic forecasting and allocation, and schema-checked narrative reports. Research and analysis only; not investment advice."),
    paths(routes::index, routes::openapi_json, routes::forecast, routes::allocate, routes::report, routes::healthz, routes::readyz, crate::metrics::serve),
    components(schemas(
        Index, ForecastRequest, AllocateRequest, Health, Problem, IntervalForecast, Allocation, Diagnostics,
        Constraints, CovarianceEstimator, ReportInput, Report, AllocationReport, PromptId, TokenUsage
    )),
    modifiers(&BearerKey)
)]
pub struct ApiDoc;

/// Declares the bearer API key the protected routes require.
#[derive(Clone, Copy, Debug)]
struct BearerKey;

impl Modify for BearerKey {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let scheme = HttpBuilder::new()
            .scheme(HttpAuthScheme::Bearer)
            .description(Some("An API key, sent as `Authorization: Bearer <key>`."))
            .build();
        openapi
            .components
            .get_or_insert_with(Default::default)
            .add_security_scheme("api_key", SecurityScheme::Http(scheme));
    }
}

/// The document as pretty-printed JSON, with the crate's version.
///
/// # Panics
///
/// Never in practice: the document is plain data and always serialises.
#[must_use]
pub fn spec_json() -> String {
    let mut doc = ApiDoc::openapi();
    doc.info.version = env!("CARGO_PKG_VERSION").to_string();
    doc.to_pretty_json().expect("the OpenAPI document serialises") + "\n"
}
