//! The performance budget: a 20-asset, 250-day allocation in under 50 ms
//! at the 95th percentile, for every method.
//!
//! Timing is only meaningful in an optimised build on a quiet machine,
//! so this runs only when `NALUFX_PERF=1` is set; CI runs it with
//! `--release` on its runner.

mod support;

use nalufx_engine::allocate::{by_name, METHODS};
use std::time::{Duration, Instant};

const BUDGET: Duration = Duration::from_millis(50);
const RUNS: usize = 100;

#[test]
fn twenty_assets_over_250_days_allocate_within_budget_at_p95() {
    if std::env::var("NALUFX_PERF").is_err() {
        eprintln!("skipped: set NALUFX_PERF=1 (and build with --release) to check the budget");
        return;
    }
    let input = support::synthetic(20, 250);
    for name in METHODS {
        let method = by_name(name, None).expect("a known method");
        let mut times: Vec<Duration> = (0..RUNS)
            .map(|_| {
                let start = Instant::now();
                let _ = method.allocate(&input).expect("the synthetic input allocates");
                start.elapsed()
            })
            .collect();
        times.sort();
        let p95 = times[RUNS * 95 / 100];
        eprintln!("{name}: p50 {:?}, p95 {p95:?}", times[RUNS / 2]);
        assert!(p95 < BUDGET, "{name}: p95 {p95:?} is over the {BUDGET:?} budget");
    }
}
