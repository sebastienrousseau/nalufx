//! Risk parity: every asset contributes the same share of the variance.

use super::{project, Allocator, Constraints, Solution};
use crate::covariance::Moments;
use crate::linalg::solve_spd;
use crate::AllocationError;
use ndarray::{Array1, Array2};

/// Equal risk contribution: `w_i (S w)_i` is the same for every asset.
///
/// Solved by Newton's method on the convex problem
/// `min 0.5 y' S y - (1/n) sum(ln y_i)` over `y > 0`, whose optimum
/// satisfies `y_i (S y)_i = 1/n`; the weights are `y / sum(y)` (Spinu,
/// 2013). If they break the bounds they are projected onto them, which
/// leaves the risk contributions only approximately equal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RiskParity;

/// Newton iterations before giving up; it converges in well under 30.
const MAX_ITERATIONS: u32 = 100;

/// Stop when every `y_i (S y)_i` is within this fraction of `1/n`.
const RELATIVE_TOLERANCE: f64 = 1e-12;

/// Newton converges quadratically to the f64 rounding floor, which
/// depends on the conditioning (from about 5e-14 to 2e-12 of `1/n` in
/// the tests), and then cannot improve. If the residual has not improved
/// for this many steps and is within [`ACCEPTABLE_TOLERANCE`], that is
/// the answer.
const STALL_STEPS: u32 = 3;

/// The loosest residual, as a fraction of `1/n`, accepted at a stall.
const ACCEPTABLE_TOLERANCE: f64 = 1e-9;

impl Allocator for RiskParity {
    fn name(&self) -> &'static str {
        "risk_parity"
    }

    fn params(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn solve(
        &self,
        moments: &Moments,
        constraints: &Constraints,
    ) -> Result<Solution, AllocationError> {
        let (y, iterations) = equal_risk_contribution(&moments.covariance)?;
        let total = y.sum();
        let erc: Vec<f64> = y.iter().map(|v| v / total).collect();
        let projected =
            erc.iter().any(|w| *w > constraints.max_weight || *w < constraints.min_weight);
        let weights = if projected { project(&erc, constraints)? } else { erc };
        Ok(Solution { weights, solver_status: "converged".to_string(), iterations, projected })
    }
}

/// The unnormalised `y` with `y_i (S y)_i = 1/n`, and the iterations used.
fn equal_risk_contribution(s: &Array2<f64>) -> Result<(Array1<f64>, u32), AllocationError> {
    let s = &normalised(s)?;
    let b = 1.0 / s.nrows() as f64;
    // Start at inverse volatility, which is the answer when uncorrelated.
    let mut y: Array1<f64> = s.diag().mapv(|v| 1.0 / v.sqrt());
    let mut progress = Progress::default();
    for iteration in 0..MAX_ITERATIONS {
        if progress.converged(residual(s, &y, b), b) {
            return Ok((y, iteration));
        }
        let (gradient, step) = newton_step(s, &y, b)?;
        y = line_search(s, &y, &step, &gradient, b);
    }
    Err(AllocationError::SolverFailed(format!(
        "risk parity did not converge in {MAX_ITERATIONS} Newton steps"
    )))
}

/// `S` divided by its average variance. Equal-risk weights do not change
/// when `S` is scaled, and daily covariances are about 1e-4, which would
/// put `y` near 100 and the rounding floor far from `1/n`.
fn normalised(s: &Array2<f64>) -> Result<Array2<f64>, AllocationError> {
    let scale = s.diag().mean().ok_or(AllocationError::EmptyInput)?;
    if !(scale.is_finite() && scale > 0.0) || s.diag().iter().any(|v| *v <= 0.0) {
        return Err(AllocationError::ZeroVolatility);
    }
    Ok(s / scale)
}

/// The gradient of the objective at `y` and the Newton step `H^-1 g`.
fn newton_step(
    s: &Array2<f64>,
    y: &Array1<f64>,
    b: f64,
) -> Result<(Array1<f64>, Array1<f64>), AllocationError> {
    let gradient = s.dot(y) - y.mapv(|v| b / v);
    let mut hessian = s.clone();
    hessian.diag_mut().zip_mut_with(y, |h, v| *h += b / (v * v));
    let step = solve_spd(&hessian, gradient.view())?;
    Ok((gradient, step))
}

/// The best residual so far and how many steps have not improved on it.
#[derive(Debug, Default)]
struct Progress {
    best: Option<f64>,
    stalled: u32,
}

impl Progress {
    /// Records a residual; whether it is good enough to stop.
    fn converged(&mut self, r: f64, b: f64) -> bool {
        if self.best.is_none_or(|best| r < best) {
            (self.best, self.stalled) = (Some(r), 0);
        } else {
            self.stalled += 1;
        }
        r <= RELATIVE_TOLERANCE * b
            || (self.stalled >= STALL_STEPS && r <= ACCEPTABLE_TOLERANCE * b)
    }
}

/// The largest of residual `max |y_i (S y)_i - b|`.
fn residual(s: &Array2<f64>, y: &Array1<f64>, b: f64) -> f64 {
    y.iter().zip(&s.dot(y)).map(|(yi, si)| (yi * si - b).abs()).fold(0.0, f64::max)
}

/// The point `y - t * step` with the largest `t` in `1, 1/2, 1/4, ...`
/// that keeps `y` positive and either decreases the objective enough
/// (Armijo) or decreases the residual. Near the optimum the objective
/// changes by about the residual squared, below its rounding, so Armijo
/// alone would reject the steps that still halve the residual's digits.
fn line_search(
    s: &Array2<f64>,
    y: &Array1<f64>,
    step: &Array1<f64>,
    gradient: &Array1<f64>,
    b: f64,
) -> Array1<f64> {
    /// Halvings before giving up on the step.
    const MAX_HALVINGS: usize = 60;
    let (start, start_residual) = (objective(s, y, b), residual(s, y, b));
    let slope = gradient.dot(step);
    let mut t = 1.0;
    for _ in 0..MAX_HALVINGS {
        let next = y - &(step * t);
        let armijo = || objective(s, &next, b) <= 1e-4f64.mul_add(-t * slope, start);
        if next.iter().all(|v| *v > 0.0) && (armijo() || residual(s, &next, b) < start_residual) {
            return next;
        }
        t *= 0.5;
    }
    y.clone()
}

/// The objective `0.5 y' S y - b sum(ln y_i)`.
fn objective(s: &Array2<f64>, y: &Array1<f64>, b: f64) -> f64 {
    0.5 * y.dot(&s.dot(y)) - b * y.iter().map(|x| x.ln()).sum::<f64>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::array;

    fn moments(covariance: Array2<f64>) -> Moments {
        let n = covariance.nrows();
        Moments { mean: Array1::zeros(n), covariance, shrinkage: None }
    }

    fn contributions(s: &Array2<f64>, w: &[f64]) -> Vec<f64> {
        let w = Array1::from(w.to_vec());
        let sw = s.dot(&w);
        let total = w.dot(&sw);
        w.iter().zip(&sw).map(|(wi, si)| wi * si / total).collect()
    }

    #[test]
    fn uncorrelated_assets_get_inverse_volatility() {
        let s = array![[1.0, 0.0], [0.0, 4.0]];
        let w = RiskParity.solve(&moments(s), &Constraints::default()).unwrap().weights;
        // Volatilities 1 and 2: weights 2/3 and 1/3.
        assert!((w[0] - 2.0 / 3.0).abs() < 1e-12 && (w[1] - 1.0 / 3.0).abs() < 1e-12, "{w:?}");
    }

    #[test]
    fn correlated_assets_get_equal_risk_contributions() {
        let s = array![[0.04, 0.006, 0.002], [0.006, 0.09, 0.018], [0.002, 0.018, 0.01]];
        let solution = RiskParity.solve(&moments(s.clone()), &Constraints::default()).unwrap();
        assert!(!solution.projected);
        for rc in contributions(&s, &solution.weights) {
            assert!((rc - 1.0 / 3.0).abs() < 1e-6, "{rc}");
        }
    }

    #[test]
    fn a_binding_cap_is_projected_and_says_so() {
        let s = array![[0.0001, 0.0], [0.0, 0.04],];
        let c = Constraints { max_weight: 0.6, min_weight: 0.0 };
        let solution = RiskParity.solve(&moments(s), &c).unwrap();
        assert!(solution.projected);
        assert!((solution.weights[0] - 0.6).abs() < 1e-12, "{:?}", solution.weights);
    }
}
