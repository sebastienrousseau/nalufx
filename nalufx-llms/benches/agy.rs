//! Benchmarks for the Agy response parser.
//!
//! Only `parse_agy_response` is measured. It is the one part of the
//! adapter that is pure and runs per response, so it is both worth
//! measuring and possible to measure honestly -- benchmarking the HTTP
//! path would time the network, not this crate.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
// criterion::black_box is deprecated in 0.8 in favour of the std one.
use nalufx_llms::llms::agy::parse_agy_response;
use std::hint::black_box;

/// Builds a response body carrying `n` whitespace-separated values in a
/// single choice.
fn body_with_values(n: usize) -> String {
    let content: Vec<String> = (0..n).map(|i| format!("{}.5", i)).collect();
    format!(r#"{{"choices":[{{"message":{{"content":"{}"}}}}]}}"#, content.join(" "))
}

/// Builds a response body spreading `n` values across `choices` choices.
fn body_with_choices(choices: usize, per_choice: usize) -> String {
    let parts: Vec<String> = (0..choices)
        .map(|c| {
            let content: Vec<String> = (0..per_choice).map(|i| format!("{}.{}", c, i)).collect();
            format!(r#"{{"message":{{"content":"{}"}}}}"#, content.join(" "))
        })
        .collect();
    format!(r#"{{"choices":[{}]}}"#, parts.join(","))
}

fn bench_by_value_count(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_agy_response/values");
    for n in [6_usize, 64, 512, 4096] {
        let body = body_with_values(n);
        group.throughput(Throughput::Bytes(body.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &body, |b, body| {
            b.iter(|| parse_agy_response(black_box(body)).expect("valid body"));
        });
    }
    group.finish();
}

fn bench_by_choice_count(c: &mut Criterion) {
    // The parser flattens across choices, so this separates the cost of
    // many choices from the cost of many values.
    let mut group = c.benchmark_group("parse_agy_response/choices");
    for choices in [1_usize, 8, 64] {
        let body = body_with_choices(choices, 16);
        group.throughput(Throughput::Bytes(body.len() as u64));
        group.bench_with_input(BenchmarkId::from_parameter(choices), &body, |b, body| {
            b.iter(|| parse_agy_response(black_box(body)).expect("valid body"));
        });
    }
    group.finish();
}

fn bench_failure_path(c: &mut Criterion) {
    // A malformed body should be cheap to reject; if this ever
    // approaches the success path, the parser is doing work before
    // validating.
    let malformed = r#"{"unexpected":true}"#;
    let _ = c.bench_function("parse_agy_response/rejects_wrong_shape", |b| {
        b.iter(|| parse_agy_response(black_box(malformed)).expect_err("wrong shape"));
    });
}

criterion_group!(benches, bench_by_value_count, bench_by_choice_count, bench_failure_path);
criterion_main!(benches);
