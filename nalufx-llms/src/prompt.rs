//! Prompts as versioned templates.
//!
//! Each prompt is a Markdown file in `nalufx-llms/prompts/`, compiled into
//! the crate, with a `name` and `version` in its front matter and a
//! `## System` and `## User` section. `{{key}}` placeholders are filled by
//! [`Prompt::render`]. Every structured reply records the prompt's name
//! and version, so a report can be traced to the exact text that asked
//! for it.

use crate::error::LlmError;
use serde::{Deserialize, Serialize};

/// A prompt's identity, recorded with every reply produced from it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct PromptId {
    /// The prompt's name.
    pub name: String,
    /// The prompt's version; it changes whenever the text does.
    pub version: u32,
}

/// A parsed prompt template.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prompt {
    /// The prompt's identity.
    pub id: PromptId,
    system: String,
    user: String,
}

/// A prompt with its placeholders filled.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rendered {
    /// The prompt it came from.
    pub id: PromptId,
    /// The system instructions.
    pub system: String,
    /// The user message.
    pub user: String,
}

/// The allocation report prompt.
pub const ALLOCATION_REPORT: &str = include_str!("../prompts/allocation_report.md");

impl Prompt {
    /// Parses a template.
    ///
    /// # Errors
    ///
    /// [`LlmError::Config`] if the front matter lacks a name or a numeric
    /// version, or a `## System` or `## User` section is missing.
    pub fn parse(source: &str) -> Result<Self, LlmError> {
        let (front, body) = front_matter(source)?;
        let (system, user) =
            sections(body).ok_or_else(|| invalid("needs a `## System` and a `## User` section"))?;
        Ok(Self { id: identity(front)?, system, user })
    }

    /// Fills every `{{key}}` placeholder. The values are inserted as they
    /// are: one containing `{{` is not read as a placeholder.
    ///
    /// # Errors
    ///
    /// [`LlmError::Config`] naming the first placeholder with no value.
    pub fn render(&self, values: &[(&str, &str)]) -> Result<Rendered, LlmError> {
        Ok(Rendered {
            id: self.id.clone(),
            system: self.fill(&self.system, values)?,
            user: self.fill(&self.user, values)?,
        })
    }

    /// One section with its placeholders filled.
    fn fill(&self, text: &str, values: &[(&str, &str)]) -> Result<String, LlmError> {
        let mut out = String::with_capacity(text.len());
        let mut rest = text;
        while let Some(start) = rest.find("{{") {
            let end = rest[start..].find("}}").map(|e| start + e).ok_or_else(|| {
                LlmError::Config(format!("prompt {}: unterminated placeholder", self.id.name))
            })?;
            let key = &rest[start + 2..end];
            let value =
                values.iter().find(|(k, _)| *k == key).map(|(_, v)| *v).ok_or_else(|| {
                    LlmError::Config(format!("prompt {}: no value for {{{{{key}}}}}", self.id.name))
                })?;
            out.push_str(&rest[..start]);
            out.push_str(value);
            rest = &rest[end + 2..];
        }
        out.push_str(rest);
        Ok(out)
    }
}

/// A template error.
fn invalid(what: &str) -> LlmError {
    LlmError::Config(format!("prompt template: {what}"))
}

/// The front matter and the body after it.
fn front_matter(source: &str) -> Result<(&str, &str), LlmError> {
    let rest = source.strip_prefix("---\n").ok_or_else(|| invalid("no front matter"))?;
    rest.split_once("\n---\n").ok_or_else(|| invalid("unterminated front matter"))
}

/// The `name:` and `version:` in the front matter.
fn identity(front: &str) -> Result<PromptId, LlmError> {
    let field = |key: &str| front.lines().find_map(|l| l.strip_prefix(key)).map(str::trim);
    let name = field("name:").filter(|n| !n.is_empty()).ok_or_else(|| invalid("no name"))?;
    let version = field("version:")
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| invalid("no numeric version"))?;
    Ok(PromptId { name: name.to_string(), version })
}

/// The trimmed `## System` and `## User` sections of a template body.
fn sections(body: &str) -> Option<(String, String)> {
    let (_, after_system) = body.split_once("## System\n")?;
    let (system, user) = after_system.split_once("## User\n")?;
    Some((system.trim().to_string(), user.trim().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: &str = "---\nname: t\nversion: 3\n---\n## System\n\nYou are {{who}}.\n\n## User\n\nSay {{what}}.\n";

    #[test]
    fn a_template_parses_and_renders() {
        let p = Prompt::parse(T).unwrap();
        assert_eq!(p.id, PromptId { name: "t".into(), version: 3 });
        let r = p.render(&[("who", "terse"), ("what", "hi")]).unwrap();
        assert_eq!((r.system.as_str(), r.user.as_str()), ("You are terse.", "Say hi."));
    }

    #[test]
    fn an_unfilled_placeholder_is_an_error() {
        let err = Prompt::parse(T).unwrap().render(&[("who", "x")]).unwrap_err();
        assert!(matches!(&err, LlmError::Config(m) if m.contains("{{what}}")), "{err}");
    }

    #[test]
    fn a_value_with_braces_is_inserted_as_it_is() {
        let r =
            Prompt::parse(T).unwrap().render(&[("who", "{{what}}"), ("what", "{{x}}")]).unwrap();
        assert_eq!((r.system.as_str(), r.user.as_str()), ("You are {{what}}.", "Say {{x}}."));
    }

    #[test]
    fn an_unterminated_placeholder_is_an_error() {
        let p =
            Prompt::parse("---\nname: t\nversion: 1\n---\n## System\n{{oops\n## User\nx").unwrap();
        assert!(matches!(p.render(&[]), Err(LlmError::Config(_))));
    }

    #[test]
    fn a_malformed_template_is_an_error() {
        for bad in [
            "no front matter",
            "---\nname: t\n",
            "---\nversion: 1\n---\n## System\na\n## User\nb",
            "---\nname: t\nversion: one\n---\n## System\na\n## User\nb",
            "---\nname: t\nversion: 1\n---\n## System\na\n",
        ] {
            assert!(matches!(Prompt::parse(bad), Err(LlmError::Config(_))), "{bad:?}");
        }
    }

    #[test]
    fn the_shipped_report_prompt_parses_and_takes_its_four_values() {
        let p = Prompt::parse(ALLOCATION_REPORT).unwrap();
        assert_eq!(p.id, PromptId { name: "allocation_report".into(), version: 2 });
        let values = [("schema", "{}"), ("portfolio", "P"), ("facts", "{}"), ("context", "none")];
        assert!(p.render(&values).is_ok());
        assert!(p.render(&values[..3]).is_err());
    }
}
