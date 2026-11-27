//! Any bytes as a returns CSV: an error or a well-formed matrix, never a
//! panic.
#![no_main]

use libfuzzer_sys::fuzz_target;
use nalufx_cli::input::returns_from;

fuzz_target!(|data: &[u8]| {
    if let Ok((assets, returns)) = returns_from(data, "fuzz.csv") {
        assert!(!assets.is_empty(), "no assets");
        assert_eq!(returns.ncols(), assets.len(), "one column per asset");
        assert!(returns.nrows() > 0, "no rows");
        assert!(returns.iter().all(|v| v.is_finite()), "a non-finite return");
    }
});
