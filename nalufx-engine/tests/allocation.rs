//! Every allocation method, held to the same contract: invariants under
//! random input, typed errors on infeasible bounds, determinism, and a
//! golden backtest on the checked-in fixture.

use nalufx_engine::allocate::{by_name, AllocationInput, Allocator, Constraints, METHODS};
use nalufx_engine::backtest::{run, BacktestConfig};
use nalufx_engine::covariance::CovarianceEstimator;
use nalufx_engine::forecast::forecast_seasonal;
use nalufx_engine::AllocationError;
use ndarray::Array2;
use proptest::prelude::*;

const FIXTURE: &str = include_str!("fixtures/assets_500.csv");

/// The 5 assets and 500 days of `tests/fixtures/assets_500.csv`.
fn fixture() -> (Vec<String>, Array2<f64>) {
    let mut lines = FIXTURE.lines();
    let assets: Vec<String> = lines.next().unwrap_or("").split(',').map(str::to_string).collect();
    let rows: Vec<Vec<f64>> =
        lines.map(|l| l.split(',').map(|x| x.parse().expect("a number")).collect()).collect();
    let flat: Vec<f64> = rows.iter().flatten().copied().collect();
    let returns = Array2::from_shape_vec((rows.len(), assets.len()), flat).expect("a full matrix");
    (assets, returns)
}

fn input(returns: Array2<f64>, constraints: Constraints) -> AllocationInput {
    let assets = (0..returns.ncols()).map(|i| format!("A{i}")).collect();
    AllocationInput { assets, returns, constraints, estimator: CovarianceEstimator::LedoitWolf }
}

fn method(name: &str) -> Box<dyn Allocator> {
    by_name(name, None).expect("a known method")
}

#[test]
fn infeasible_bounds_are_a_typed_error_for_every_method() {
    let (_, returns) = fixture();
    let capped = Constraints { max_weight: 0.15, min_weight: 0.0 };
    let floored = Constraints { max_weight: 1.0, min_weight: 0.25 };
    for name in METHODS {
        for c in [capped, floored] {
            let result = method(name).allocate(&input(returns.clone(), c));
            assert!(matches!(result, Err(AllocationError::Infeasible(_))), "{name}: {result:?}");
        }
    }
}

#[test]
fn every_method_records_how_it_was_produced() {
    let (assets, returns) = fixture();
    for name in METHODS {
        let a = method(name)
            .allocate(&AllocationInput {
                assets: assets.clone(),
                returns: returns.clone(),
                constraints: Constraints::default(),
                estimator: CovarianceEstimator::LedoitWolf,
            })
            .expect("the fixture allocates");
        assert_eq!(a.method, name);
        assert_eq!(a.assets, assets);
        assert_eq!(a.params["estimator"], "ledoit_wolf");
        assert!(a.diagnostics.condition_number >= 1.0, "{name}");
        assert!(a.diagnostics.shrinkage.is_some(), "{name}");
    }
}

#[test]
fn methods_beyond_equal_weight_lower_the_volatility_of_a_calm_mix() {
    // BOND is far calmer than SMALL: every risk-based method holds more of it.
    let (assets, returns) = fixture();
    let bond = assets.iter().position(|a| a == "BOND").unwrap();
    let small = assets.iter().position(|a| a == "SMALL").unwrap();
    for name in ["min_variance", "risk_parity", "hrp"] {
        let w =
            method(name).allocate(&input(returns.clone(), Constraints::default())).unwrap().weights;
        assert!(w[bond] > w[small], "{name}: {w:?}");
    }
}

/// Each method's backtest on the fixture is a golden file in
/// `tests/fixtures/golden/`. A change to any figure fails here; after
/// reviewing it, run with `UPDATE_GOLDEN=1` to record the new report.
#[test]
fn backtests_match_their_golden_reports() {
    let (assets, returns) = fixture();
    let constraints = Constraints { max_weight: 0.4, min_weight: 0.0 };
    let config = BacktestConfig::default();
    let mut changed = Vec::new();
    for name in METHODS {
        let report = run(
            method(name).as_ref(),
            &assets,
            &returns,
            constraints,
            CovarianceEstimator::LedoitWolf,
            config,
        )
        .expect("the fixture backtests");
        let json = serde_json::to_string_pretty(&report).expect("a report serialises") + "\n";
        let path = format!("{}/tests/fixtures/golden/{name}.json", env!("CARGO_MANIFEST_DIR"));
        if std::env::var("UPDATE_GOLDEN").is_ok() {
            std::fs::write(&path, &json).expect("the golden file is writable");
        } else if std::fs::read_to_string(&path).ok().as_deref() != Some(json.as_str()) {
            changed.push(name);
        }
    }
    assert!(
        changed.is_empty(),
        "backtest reports changed for {changed:?}; review, then UPDATE_GOLDEN=1"
    );
}

#[test]
fn a_weekly_cycle_is_forecast_as_a_cycle() {
    let cycle = [0.0, 40.0, 10.0, -20.0, 60.0];
    let data: Vec<f64> = (0..60).map(|i| 1000.0 + i as f64 + cycle[i % 5]).collect();
    let forecast = forecast_seasonal(&data, &[5], 5).expect("a seasonal series forecasts");
    let peak = forecast.point.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).unwrap().0;
    // Day 60 is position 0 of the cycle, so the peak (position 4) is the 5th day.
    assert_eq!(peak, 4, "{:?}", forecast.point);
    assert!(forecast
        .lower
        .iter()
        .zip(&forecast.point)
        .zip(&forecast.upper)
        .all(|((l, p), u)| l <= p && p <= u));
}

#[test]
fn a_seasonal_series_too_short_for_two_cycles_is_rejected() {
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
    assert!(matches!(forecast_seasonal(&data, &[5], 2), Err(AllocationError::InvalidParameter(_))));
}

/// Proptest cases per property: 10,000, or `PROPTEST_CASES` if set.
fn cases() -> u32 {
    std::env::var("PROPTEST_CASES").ok().and_then(|v| v.parse().ok()).unwrap_or(10_000)
}

/// A returns matrix of 2 to 8 assets over 30 to 80 periods, each asset
/// with its own scale so the covariance is not near the identity.
fn returns() -> impl Strategy<Value = Array2<f64>> {
    (2usize..=8, 30usize..=80).prop_flat_map(|(n, t)| {
        (prop::collection::vec(-1.0f64..1.0, n * t), prop::collection::vec(0.002f64..0.03, n))
            .prop_map(move |(z, scale)| {
                Array2::from_shape_fn((t, n), |(i, j)| z[i * n + j] * scale[j])
            })
    })
}

/// Bounds that some weights over `n` assets satisfy.
fn constraints(n: usize) -> impl Strategy<Value = Constraints> {
    let nf = n as f64;
    (0.0f64..1.0, 0.0f64..1.0).prop_map(move |(a, b)| {
        let max_weight = 1.0 / nf + a * (1.0 - 1.0 / nf);
        Constraints { max_weight, min_weight: b * (1.0 / nf).min(max_weight) }
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(cases()))]

    /// Every weight is within its bounds (so non-negative and under the
    /// cap) and they sum to 1 within 1e-9, for every method.
    #[test]
    fn every_method_meets_the_bounds_and_the_budget(
        (r, c) in returns().prop_flat_map(|r| { let n = r.ncols(); (Just(r), constraints(n)) })
    ) {
        for name in METHODS {
            match method(name).allocate(&input(r.clone(), c)) {
                Ok(a) => {
                    let total: f64 = a.weights.iter().sum();
                    prop_assert!((total - 1.0).abs() <= 1e-9, "{}: sum {}", name, total);
                    for w in &a.weights {
                        prop_assert!(*w >= 0.0 && *w >= c.min_weight - 1e-12 && *w <= c.max_weight + 1e-12,
                            "{}: {} outside [{}, {}]", name, w, c.min_weight, c.max_weight);
                    }
                }
                Err(e) => prop_assert!(false, "{} failed on feasible input: {}", name, e),
            }
        }
    }

    /// The same input gives bit-identical weights.
    #[test]
    fn every_method_is_deterministic(r in returns()) {
        for name in METHODS {
            let first = method(name).allocate(&input(r.clone(), Constraints::default())).map(|a| a.weights);
            let second = method(name).allocate(&input(r.clone(), Constraints::default())).map(|a| a.weights);
            prop_assert_eq!(first, second, "{}", name);
        }
    }
}
