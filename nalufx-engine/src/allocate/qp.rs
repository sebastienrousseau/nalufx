//! Minimum variance and mean-variance, as quadratic programmes solved
//! with clarabel, a pure-Rust interior-point solver.

use super::{project, Allocator, Constraints, Solution};
use crate::covariance::Moments;
use crate::AllocationError;
use clarabel::algebra::CscMatrix;
use clarabel::solver::{DefaultSettings, DefaultSolver, IPSolver, SolverStatus, SupportedConeT};
use ndarray::Array2;

/// The portfolio with the least variance: minimise `w' S w` subject to
/// the bounds and a budget of 1. Expected returns are not used.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MinVariance;

impl Allocator for MinVariance {
    fn name(&self) -> &'static str {
        "min_variance"
    }

    fn params(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    fn solve(
        &self,
        moments: &Moments,
        constraints: &Constraints,
    ) -> Result<Solution, AllocationError> {
        let n = moments.mean.len();
        solve_qp(&moments.covariance, &vec![0.0; n], constraints)
    }
}

/// The Markowitz portfolio: maximise `mu' w - (lambda / 2) w' S w`
/// subject to the bounds and a budget of 1. A larger risk aversion
/// `lambda` moves it towards minimum variance; a smaller one towards the
/// highest-mean asset.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeanVariance {
    risk_aversion: f64,
}

impl MeanVariance {
    /// The risk aversion used when none is given.
    pub const DEFAULT_RISK_AVERSION: f64 = 10.0;

    /// A mean-variance allocator with this risk aversion.
    ///
    /// # Errors
    ///
    /// [`AllocationError::InvalidParameter`] unless `risk_aversion` is
    /// finite and positive.
    pub fn new(risk_aversion: f64) -> Result<Self, AllocationError> {
        if risk_aversion.is_finite() && risk_aversion > 0.0 {
            Ok(Self { risk_aversion })
        } else {
            Err(AllocationError::InvalidParameter(format!(
                "risk aversion must be finite and positive, not {risk_aversion}"
            )))
        }
    }

    /// The risk aversion.
    #[must_use]
    pub const fn risk_aversion(&self) -> f64 {
        self.risk_aversion
    }
}

impl Allocator for MeanVariance {
    fn name(&self) -> &'static str {
        "mean_variance"
    }

    fn params(&self) -> serde_json::Value {
        serde_json::json!({ "risk_aversion": self.risk_aversion })
    }

    fn solve(
        &self,
        moments: &Moments,
        constraints: &Constraints,
    ) -> Result<Solution, AllocationError> {
        let p = &moments.covariance * self.risk_aversion;
        let q: Vec<f64> = moments.mean.iter().map(|m| -m).collect();
        solve_qp(&p, &q, constraints)
    }
}

/// Minimises `0.5 w' P w + q' w` subject to `sum(w) = 1` and
/// `min <= w <= max`.
///
/// The solver's answer meets the constraints to its tolerance (about
/// 1e-8); it is then projected onto them exactly, which moves it by no
/// more than that tolerance.
fn solve_qp(
    p: &Array2<f64>,
    q: &[f64],
    constraints: &Constraints,
) -> Result<Solution, AllocationError> {
    let n = q.len();
    let (a, b) = constraint_matrix(n, constraints);
    let cones = [SupportedConeT::ZeroConeT(1), SupportedConeT::NonnegativeConeT(2 * n)];
    let settings = DefaultSettings { verbose: false, ..DefaultSettings::default() };
    let mut solver = DefaultSolver::new(&upper_triangle(p), q, &a, &b, &cones, settings)
        .map_err(|e| AllocationError::SolverFailed(e.to_string()))?;
    solver.solve();
    let status = solver.solution.status;
    match status {
        SolverStatus::Solved | SolverStatus::AlmostSolved => Ok(Solution {
            weights: project(&solver.solution.x, constraints)?,
            solver_status: status.to_string(),
            iterations: solver.solution.iterations,
            projected: false,
        }),
        SolverStatus::PrimalInfeasible | SolverStatus::AlmostPrimalInfeasible => Err(
            AllocationError::Infeasible(format!("the solver found no feasible weights ({status})")),
        ),
        other => Err(AllocationError::SolverFailed(other.to_string())),
    }
}

/// The constraints as `A w + s = b` with `s` in the cones: the first row
/// is the budget (`s = 0`), then `w <= max` and `-w <= -min` (`s >= 0`).
fn constraint_matrix(n: usize, c: &Constraints) -> (CscMatrix<f64>, Vec<f64>) {
    let mut colptr = Vec::with_capacity(n + 1);
    let mut rowval = Vec::with_capacity(3 * n);
    let mut nzval = Vec::with_capacity(3 * n);
    for j in 0..n {
        colptr.push(rowval.len());
        rowval.extend([0, 1 + j, 1 + n + j]);
        nzval.extend([1.0, 1.0, -1.0]);
    }
    colptr.push(rowval.len());
    (CscMatrix::new(1 + 2 * n, n, colptr, rowval, nzval), constraint_bounds(n, c))
}

/// The right-hand side: the budget, then `n` caps, then `n` negated floors.
fn constraint_bounds(n: usize, c: &Constraints) -> Vec<f64> {
    std::iter::once(1.0)
        .chain(std::iter::repeat_n(c.max_weight, n))
        .chain(std::iter::repeat_n(-c.min_weight, n))
        .collect()
}

/// The upper triangle of a symmetric matrix, in compressed sparse column
/// form, as clarabel takes its quadratic term.
fn upper_triangle(p: &Array2<f64>) -> CscMatrix<f64> {
    let n = p.nrows();
    let mut colptr = Vec::with_capacity(n + 1);
    let mut rowval = Vec::new();
    let mut nzval = Vec::new();
    for j in 0..n {
        colptr.push(rowval.len());
        for i in 0..=j {
            rowval.push(i);
            nzval.push(p[[i, j]]);
        }
    }
    colptr.push(rowval.len());
    CscMatrix::new(n, n, colptr, rowval, nzval)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::{array, Array1};

    fn moments(mean: Array1<f64>, covariance: Array2<f64>) -> Moments {
        Moments { mean, covariance, shrinkage: None }
    }

    /// Two uncorrelated assets with variances 1 and 4: the weights are
    /// proportional to the inverse variances, 0.8 and 0.2.
    #[test]
    fn min_variance_known_answer() {
        let m = moments(array![0.0, 0.0], array![[1.0, 0.0], [0.0, 4.0]]);
        let s = MinVariance.solve(&m, &Constraints::default()).unwrap();
        assert!(
            (s.weights[0] - 0.8).abs() < 1e-7 && (s.weights[1] - 0.2).abs() < 1e-7,
            "{:?}",
            s.weights
        );
        assert_eq!(s.solver_status, "Solved");
    }

    #[test]
    fn min_variance_respects_a_cap() {
        let m = moments(
            array![0.0, 0.0, 0.0],
            array![[1.0, 0.0, 0.0], [0.0, 4.0, 0.0], [0.0, 0.0, 9.0]],
        );
        let c = Constraints { max_weight: 0.5, min_weight: 0.0 };
        let w = MinVariance.solve(&m, &c).unwrap().weights;
        assert!(w.iter().all(|x| *x <= 0.5 && *x >= 0.0), "{w:?}");
        assert!((w[0] - 0.5).abs() < 1e-7, "{w:?}");
        // The rest is split 9:4 between variances 4 and 9: 0.5 * 9/13 and 0.5 * 4/13.
        assert!((w[1] - 0.5 * 9.0 / 13.0).abs() < 1e-6, "{w:?}");
    }

    #[test]
    fn mean_variance_tilts_towards_the_higher_mean() {
        let cov = array![[0.04, 0.0], [0.0, 0.04]];
        let m = moments(array![0.10, 0.05], cov);
        let w =
            MeanVariance::new(10.0).unwrap().solve(&m, &Constraints::default()).unwrap().weights;
        // Unconstrained optimum: w0 - w1 = (0.10 - 0.05) / (10 * 0.04 * 2) * 2 = 0.125 -> 0.5625 / 0.4375.
        assert!((w[0] - 0.5625).abs() < 1e-6 && (w[1] - 0.4375).abs() < 1e-6, "{w:?}");
    }

    #[test]
    fn mean_variance_rejects_a_bad_risk_aversion() {
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(matches!(MeanVariance::new(bad), Err(AllocationError::InvalidParameter(_))));
        }
    }

    #[test]
    fn the_constraint_matrix_has_the_budget_and_both_bounds() {
        let (a, b) = constraint_matrix(2, &Constraints { max_weight: 0.7, min_weight: 0.1 });
        assert_eq!((a.m, a.n), (5, 2));
        assert_eq!(b, vec![1.0, 0.7, 0.7, -0.1, -0.1]);
    }
}
