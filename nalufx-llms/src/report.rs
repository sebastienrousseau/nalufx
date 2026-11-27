//! Narrative reports about an allocation, as schema-checked JSON.
//!
//! The model writes words only. It is given the computed figures and a
//! JSON Schema generated from [`AllocationReport`]; its reply must
//! deserialise into that type exactly (no missing or unknown fields) and
//! pass [`AllocationReport::check`], or the call fails. There is no
//! fallback text.

use crate::error::LlmError;
use crate::prompt::{Prompt, PromptId, ALLOCATION_REPORT};
use crate::provider::{ChatRequest, LlmProvider, TokenUsage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The schema's name in errors.
pub const SCHEMA: &str = "allocation_report";

/// The most items in a list section.
const MAX_ITEMS: usize = 8;

/// A narrative report about an allocation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct AllocationReport {
    /// Two to four sentences on what the allocation does.
    pub summary: String,
    /// Why the allocation looks the way it does, one point per item (1 to 8).
    pub rationale: Vec<String>,
    /// What could go wrong, one risk per item (1 to 8).
    pub risks: Vec<String>,
    /// That the report is for research and analysis, not advice.
    pub disclaimer: String,
}

impl AllocationReport {
    /// Checks what the schema cannot say: no blank text, and 1 to 8 items
    /// in each list.
    ///
    /// # Errors
    ///
    /// A description of the first problem.
    pub fn check(&self) -> Result<(), String> {
        if self.summary.trim().is_empty() || self.disclaimer.trim().is_empty() {
            return Err("the summary and the disclaimer must not be blank".to_string());
        }
        for (name, items) in [("rationale", &self.rationale), ("risks", &self.risks)] {
            if !(1..=MAX_ITEMS).contains(&items.len()) || items.iter().any(|i| i.trim().is_empty())
            {
                return Err(format!("{name} needs 1 to {MAX_ITEMS} non-blank items"));
            }
        }
        Ok(())
    }
}

/// What a report is written from.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct ReportInput {
    /// The portfolio's name.
    pub portfolio: String,
    /// The computed figures (weights, method, backtest statistics), as JSON.
    #[cfg_attr(feature = "openapi", schema(value_type = Object))]
    pub facts: serde_json::Value,
    /// The investor's stated values and objectives, in their words.
    pub context: String,
}

/// A validated report and how it was produced.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Report {
    /// The report.
    pub report: AllocationReport,
    /// The prompt that asked for it.
    pub prompt: PromptId,
    /// The provider that wrote it.
    pub provider: String,
    /// The model that wrote it.
    pub model: String,
    /// Tokens used.
    pub usage: TokenUsage,
}

/// Asks the provider for a report and validates the reply.
///
/// # Errors
///
/// - [`LlmError::Schema`] if the reply is not one JSON object matching
///   [`AllocationReport`], or fails [`AllocationReport::check`].
/// - Any error from the provider.
pub async fn generate(provider: &dyn LlmProvider, input: &ReportInput) -> Result<Report, LlmError> {
    let prompt = Prompt::parse(ALLOCATION_REPORT)?;
    let schema = serde_json::to_string_pretty(&schemars::schema_for!(AllocationReport))
        .map_err(|e| LlmError::Config(format!("the report schema did not serialise: {e}")))?;
    let facts = serde_json::to_string_pretty(&input.facts)
        .map_err(|e| LlmError::Config(format!("the report facts did not serialise: {e}")))?;
    let rendered = prompt.render(&[
        ("schema", &schema),
        ("portfolio", &input.portfolio),
        ("facts", &facts),
        ("context", &input.context),
    ])?;
    let mut request = ChatRequest::user(Some(&rendered.system), rendered.user);
    request.json = true;
    let response = provider.complete(&request).await?;
    let report = parse(provider.name(), &response.text)?;
    Ok(Report {
        report,
        prompt: rendered.id,
        provider: response.provider,
        model: response.model,
        usage: response.usage,
    })
}

/// The reply as a checked report. A reply fenced as a ```json code block
/// is accepted; anything else around the object is not.
///
/// # Errors
///
/// [`LlmError::Schema`] with what did not match.
pub fn parse(provider: &'static str, text: &str) -> Result<AllocationReport, LlmError> {
    let schema_error = |message: String| LlmError::Schema { provider, schema: SCHEMA, message };
    let report: AllocationReport =
        serde_json::from_str(unfenced(text)).map_err(|e| schema_error(e.to_string()))?;
    report.check().map_err(schema_error)?;
    Ok(report)
}

/// The text inside a ```json (or plain ```) fence, or the text itself.
fn unfenced(text: &str) -> &str {
    let t = text.trim();
    t.strip_prefix("```json")
        .or_else(|| t.strip_prefix("```"))
        .and_then(|inner| inner.strip_suffix("```"))
        .map_or(t, str::trim)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"{"summary":"Mostly bonds.","rationale":["Bonds are calmer."],"risks":["Rates rise."],"disclaimer":"Research only, not advice."}"#;

    #[test]
    fn a_valid_reply_parses() {
        let r = parse("openai", VALID).unwrap();
        assert_eq!(r.rationale, vec!["Bonds are calmer."]);
    }

    #[test]
    fn a_fenced_reply_parses() {
        assert!(parse("openai", &format!("```json\n{VALID}\n```")).is_ok());
        assert!(parse("openai", &format!("```\n{VALID}\n```")).is_ok());
    }

    #[test]
    fn prose_around_the_object_is_a_schema_error() {
        let err = parse("anthropic", &format!("Here you go: {VALID}")).unwrap_err();
        assert!(matches!(err, LlmError::Schema { provider: "anthropic", schema: SCHEMA, .. }));
    }

    #[test]
    fn missing_unknown_and_blank_fields_are_schema_errors() {
        let missing = r#"{"summary":"s","rationale":["r"],"risks":["k"]}"#;
        let unknown =
            r#"{"summary":"s","rationale":["r"],"risks":["k"],"disclaimer":"d","weights":[0.5]}"#;
        let blank = r#"{"summary":" ","rationale":["r"],"risks":["k"],"disclaimer":"d"}"#;
        let empty_list = r#"{"summary":"s","rationale":[],"risks":["k"],"disclaimer":"d"}"#;
        let long_list = format!(
            r#"{{"summary":"s","rationale":{},"risks":["k"],"disclaimer":"d"}}"#,
            serde_json::json!(vec!["x"; 9])
        );
        for bad in [missing, unknown, blank, empty_list, long_list.as_str()] {
            assert!(matches!(parse("gemini", bad), Err(LlmError::Schema { .. })), "{bad}");
        }
    }

    #[test]
    fn the_schema_names_every_field() {
        let schema = serde_json::to_value(schemars::schema_for!(AllocationReport)).unwrap();
        let required = schema["required"].as_array().unwrap();
        for field in ["summary", "rationale", "risks", "disclaimer"] {
            assert!(required.iter().any(|r| r == field), "{field}");
        }
        assert_eq!(schema["additionalProperties"], false);
    }
}
