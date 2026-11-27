//! Every method on arbitrary returns and bounds: an error, or weights that
//! are finite, sum to 1 and respect the bounds; never a panic.
#![no_main]

use libfuzzer_sys::fuzz_target;
use nalufx_engine::allocate::{by_name, AllocationInput, Constraints, METHODS};
use nalufx_engine::covariance::CovarianceEstimator;
use ndarray::Array2;

/// The input decoded from bytes: a header of shape, bounds, method and
/// estimator, then the returns as little-endian f64s (any bit pattern,
/// so NaN and infinity occur).
fn decode(data: &[u8]) -> Option<(usize, AllocationInput)> {
    let (head, body) = data.split_first_chunk::<6>()?;
    let n = usize::from(head[0] % 8) + 1;
    let t = usize::from(head[1] % 40) + 2;
    let method = usize::from(head[2]) % METHODS.len();
    let max_weight = f64::from(head[3]) / 255.0;
    let min_weight = f64::from(head[4]) / 255.0 * max_weight;
    let estimator =
        if head[5] % 2 == 0 { CovarianceEstimator::LedoitWolf } else { CovarianceEstimator::Sample };
    let mut values = body.chunks_exact(8).map(|c| f64::from_le_bytes(c.try_into().unwrap()));
    let returns = Array2::from_shape_fn((t, n), |_| values.next().unwrap_or(0.0));
    let input = AllocationInput {
        assets: (0..n).map(|i| format!("A{i}")).collect(),
        returns,
        constraints: Constraints { max_weight, min_weight },
        estimator,
    };
    Some((method, input))
}

fuzz_target!(|data: &[u8]| {
    let Some((method, input)) = decode(data) else { return };
    let Ok(allocator) = by_name(METHODS[method], Some(2.0)) else { return };
    if let Ok(allocation) = allocator.allocate(&input) {
        let w = &allocation.weights;
        assert!(w.iter().all(|x| x.is_finite()), "{}: {w:?}", METHODS[method]);
        let total: f64 = w.iter().sum();
        assert!((total - 1.0).abs() < 1e-6, "{}: weights sum to {total}", METHODS[method]);
        let (lo, hi) = (input.constraints.min_weight - 1e-6, input.constraints.max_weight + 1e-6);
        assert!(w.iter().all(|x| (lo..=hi).contains(x)), "{}: {w:?} outside {:?}", METHODS[method], input.constraints);
    }
});
