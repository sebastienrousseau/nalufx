//! Allocation time for 20 assets over 250 days, per method.

#[path = "../tests/support/mod.rs"]
mod support;

use criterion::{criterion_group, criterion_main, Criterion};
use nalufx_engine::allocate::{by_name, METHODS};
use std::hint::black_box;

/// Times each method on the same synthetic input.
fn allocate(c: &mut Criterion) {
    let input = support::synthetic(20, 250);
    let mut group = c.benchmark_group("allocate_20x250");
    for name in METHODS {
        let method = by_name(name, None).expect("a known method");
        let _ = group.bench_function(name, |b| b.iter(|| method.allocate(black_box(&input))));
    }
    group.finish();
}

criterion_group!(benches, allocate);
criterion_main!(benches);
