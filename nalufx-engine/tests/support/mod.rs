//! Shared synthetic data for the performance tests and benchmarks.

use nalufx_engine::allocate::{AllocationInput, Constraints};
use nalufx_engine::covariance::CovarianceEstimator;
use ndarray::Array2;

/// A deterministic one-factor returns matrix of `assets` columns over
/// `days` rows, from a fixed-seed linear congruential generator.
pub(crate) fn synthetic(assets: usize, days: usize) -> AllocationInput {
    let mut state: u64 = 20_261_127;
    let mut uniform = move || {
        state =
            state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        (state >> 11) as f64 / (1u64 << 53) as f64 - 0.5
    };
    let mut returns = Array2::<f64>::zeros((days, assets));
    for mut row in returns.rows_mut() {
        let market = 0.02 * uniform();
        for (j, r) in row.iter_mut().enumerate() {
            let beta = 0.2 + 1.2 * j as f64 / assets as f64;
            *r = 0.0003 + beta * market + 0.02 * uniform();
        }
    }
    AllocationInput {
        assets: (0..assets).map(|i| format!("A{i}")).collect(),
        returns,
        constraints: Constraints { max_weight: 0.2, min_weight: 0.0 },
        estimator: CovarianceEstimator::LedoitWolf,
    }
}
