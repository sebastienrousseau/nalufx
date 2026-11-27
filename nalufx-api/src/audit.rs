//! The audit log: one JSON line per allocation and per report.
//!
//! An allocation's line holds everything needed to reproduce it: the
//! canonical input (the request as JSON, fields in a fixed order) and its
//! SHA-256, the method and its parameters, the engine version and the
//! weights. Running the input through the same engine version gives the
//! same weights, bit for bit. A report's line holds the SHA-256 of its
//! input, the prompt's name and version, the provider, the model and the
//! tokens, never the investor's words or the model's reply.
//!
//! Lines go to an [`AuditSink`]. The one here appends to a file; the
//! trait is the seam for sending them elsewhere. Writing is required:
//! when a line cannot be written, the request fails with 503, so no
//! answer goes out without its record.

use crate::app::AppState;
use crate::auth::KeyId;
use crate::problem::Problem;
use crate::routes::AllocateRequest;
use axum::http::{HeaderMap, StatusCode};
use nalufx_engine::allocate::Allocation;
use nalufx_llms::report::{Report, ReportInput};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// The engine version stamped on every line.
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Where audit lines go.
pub trait AuditSink: Send + Sync + std::fmt::Debug {
    /// Writes one record.
    ///
    /// # Errors
    ///
    /// A description of why it could not be written.
    fn record(&self, entry: &AuditEntry) -> Result<(), String>;
}

/// One audit record.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuditEntry {
    /// When, as RFC 3339 UTC.
    pub time: String,
    /// `allocation` or `report`.
    pub kind: String,
    /// The request's id (`x-request-id`).
    pub request_id: String,
    /// The id of the key that made the request.
    pub key_id: String,
    /// SHA-256 of `input`, in hex.
    pub input_sha256: String,
    /// The canonical input: an allocation's request. Empty for a report,
    /// whose input holds the investor's words.
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub input: serde_json::Value,
    /// The engine version that answered.
    pub engine_version: String,
    /// What came out: an allocation's method, parameters and weights, or
    /// a report's prompt, provider, model and tokens.
    pub output: serde_json::Value,
}

impl AuditEntry {
    /// A record stamped now, with the SHA-256 of `canonical`.
    #[must_use]
    pub fn new(kind: &str, request_id: &str, key_id: &str, canonical: &[u8]) -> Self {
        Self {
            time: rfc3339(SystemTime::now()),
            kind: kind.to_string(),
            request_id: request_id.to_string(),
            key_id: key_id.to_string(),
            input_sha256: hex::encode(Sha256::digest(canonical)),
            input: serde_json::Value::Null,
            engine_version: ENGINE_VERSION.to_string(),
            output: serde_json::Value::Null,
        }
    }
}

/// Appends each record as one line to a file, opened for appending only.
#[derive(Debug)]
pub struct FileSink {
    file: Mutex<File>,
}

impl FileSink {
    /// Opens (or creates) `path` for appending.
    ///
    /// # Errors
    ///
    /// If the file cannot be opened.
    pub fn open(path: &Path) -> Result<Self, String> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| format!("the audit log {} cannot be opened: {e}", path.display()))?;
        Ok(Self { file: Mutex::new(file) })
    }
}

impl AuditSink for FileSink {
    fn record(&self, entry: &AuditEntry) -> Result<(), String> {
        let mut line = serde_json::to_vec(entry).map_err(|e| e.to_string())?;
        line.push(b'\n');
        let mut file =
            self.file.lock().map_err(|_| "the audit log lock is poisoned".to_string())?;
        // One write per line, so concurrent writers never interleave.
        file.write_all(&line).and_then(|()| file.flush()).map_err(|e| e.to_string())
    }
}

/// A time as RFC 3339 UTC, to the second: `2026-11-27T16:04:05Z`.
#[must_use]
pub fn rfc3339(time: SystemTime) -> String {
    let secs = time.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let (days, rest) = (secs / 86_400, secs % 86_400);
    let (y, m, d) = civil(i64::try_from(days).unwrap_or(0));
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rest / 3600, rest % 3600 / 60, rest % 60)
}

/// The calendar date of a day count since 1970-01-01 (Howard Hinnant's
/// `civil_from_days`).
const fn civil(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    (year, m, d)
}

/// Who made a request: its id and its key's id.
#[derive(Clone, Copy, Debug)]
pub struct Caller<'a> {
    request_id: &'a str,
    key_id: &'a str,
}

impl<'a> Caller<'a> {
    /// The caller of a request with these headers, authenticated as `key`.
    #[must_use]
    pub fn new(key: &'a KeyId, headers: &'a HeaderMap) -> Self {
        let request_id =
            headers.get(crate::problem::REQUEST_ID).and_then(|v| v.to_str().ok()).unwrap_or("");
        Self { request_id, key_id: &key.0 }
    }
}

/// Writes an allocation's audit line, with its canonical input, if there
/// is an audit log.
///
/// # Errors
///
/// A 503 problem if the line cannot be written.
pub fn audit_allocation(
    state: &AppState,
    caller: &Caller<'_>,
    request: &AllocateRequest,
    allocation: &Allocation,
) -> Result<(), Problem> {
    let Some(sink) = &state.audit else { return Ok(()) };
    let canonical = serde_json::to_vec(request).map_err(|e| audit_failed(&e.to_string()))?;
    let mut entry = AuditEntry::new("allocation", caller.request_id, caller.key_id, &canonical);
    entry.input = serde_json::to_value(request).map_err(|e| audit_failed(&e.to_string()))?;
    entry.output = serde_json::json!({
        "method": allocation.method,
        "params": allocation.params,
        "assets": allocation.assets,
        "weights": allocation.weights,
    });
    sink.record(&entry).map_err(|e| audit_failed(&e))
}

/// Writes a report's audit line: the SHA-256 of its input, never the
/// investor's words or the reply.
///
/// # Errors
///
/// A 503 problem if the line cannot be written.
pub fn audit_report(
    state: &AppState,
    caller: &Caller<'_>,
    input: &ReportInput,
    report: &Report,
) -> Result<(), Problem> {
    let Some(sink) = &state.audit else { return Ok(()) };
    let canonical = serde_json::to_vec(input).map_err(|e| audit_failed(&e.to_string()))?;
    let mut entry = AuditEntry::new("report", caller.request_id, caller.key_id, &canonical);
    entry.output = serde_json::json!({
        "prompt": report.prompt,
        "provider": report.provider,
        "model": report.model,
        "usage": report.usage,
    });
    sink.record(&entry).map_err(|e| audit_failed(&e))
}

/// A 503 for an audit line that could not be written: no answer goes out
/// without its record.
fn audit_failed(reason: &str) -> Problem {
    tracing::error!(reason, "the audit log could not be written");
    Problem::new(StatusCode::SERVICE_UNAVAILABLE, "the audit log could not be written")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn times_are_rfc3339_utc() {
        assert_eq!(rfc3339(UNIX_EPOCH), "1970-01-01T00:00:00Z");
        // date -u -d @1795795445 +%FT%TZ
        assert_eq!(
            rfc3339(UNIX_EPOCH + Duration::from_secs(1_795_795_445)),
            "2026-11-27T16:04:05Z"
        );
        // A leap day: date -u -d 2024-02-29T23:59:59Z +%s
        assert_eq!(
            rfc3339(UNIX_EPOCH + Duration::from_secs(1_709_251_199)),
            "2024-02-29T23:59:59Z"
        );
    }

    #[test]
    fn a_file_sink_appends_one_line_per_record() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("audit.jsonl");
        std::fs::write(&path, "earlier line\n").unwrap();
        let sink = FileSink::open(&path).unwrap();
        let entry = AuditEntry::new("allocation", "req-1", "9f86d081", b"{}");
        sink.record(&entry).unwrap();
        sink.record(&entry).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[0], "earlier line", "an existing log is appended to, never truncated");
        assert_eq!(lines.len(), 3);
        let back: AuditEntry = serde_json::from_str(lines[1]).unwrap();
        assert_eq!(back, entry);
        // printf %s '{}' | sha256sum
        assert_eq!(
            back.input_sha256,
            "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"
        );
    }
}
