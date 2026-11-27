//! Equal weight: the baseline every other method must beat.

use super::{Allocator, Constraints, Solution};
use crate::covariance::Moments;
use crate::AllocationError;

/// `1 / n` to every asset. It ignores the moments entirely, and it is
/// within any bounds that pass [`Constraints::check`], since `n * max >= 1`
/// and `n * min <= 1`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EqualWeight;

impl Allocator for EqualWeight {
    fn name(&self) -> &'static str {
        "equal_weight"
    }

    fn params(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn solve(&self, moments: &Moments, _: &Constraints) -> Result<Solution, AllocationError> {
        let n = moments.mean.len();
        if n == 0 {
            return Err(AllocationError::EmptyInput);
        }
        Ok(Solution {
            weights: vec![1.0 / n as f64; n],
            solver_status: "closed_form".to_string(),
            iterations: 0,
            projected: false,
        })
    }
}
