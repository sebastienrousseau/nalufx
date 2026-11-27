//! Any text as the investor's words, and as a model's reply: the cleaned
//! words carry nothing that could escape the prompt's tags or hide text,
//! and parsing a reply returns a report or a schema error, never a panic.
#![no_main]

use libfuzzer_sys::fuzz_target;
use nalufx_llms::report::{parse, words};

fuzz_target!(|text: &str| {
    if let Ok(clean) = words("context", text, 2_000) {
        assert!(!clean.contains('<') && !clean.contains('>'), "{clean:?}");
        assert!(!clean.chars().any(|c| c.is_control() && c != '\n' && c != '\t'), "{clean:?}");
        assert!(!clean.contains('\u{202e}') && !clean.contains('\u{200b}'), "{clean:?}");
        assert!(clean.chars().count() <= text.chars().count());
    } else {
        assert!(text.chars().count() > 2_000);
    }
    if let Ok(report) = parse("fuzz", text) {
        assert_eq!(report.check(), Ok(()), "a parsed reply failed its own check");
    }
});
