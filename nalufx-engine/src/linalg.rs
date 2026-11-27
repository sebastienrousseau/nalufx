//! Small dense linear algebra for covariance matrices.
//!
//! Portfolio problems have tens of assets, so plain `O(n^3)` routines on
//! `ndarray` are fast enough and keep the engine free of a BLAS or
//! LAPACK dependency. Every routine is deterministic.

use crate::AllocationError;
use ndarray::{Array1, Array2, ArrayView1};

/// The lower Cholesky factor `L` of a symmetric positive-definite matrix,
/// with `a = L L^T`.
///
/// # Errors
///
/// [`AllocationError::InvalidData`] if `a` is not square, or not positive
/// definite (a pivot is zero, negative or not finite).
pub fn cholesky(a: &Array2<f64>) -> Result<Array2<f64>, AllocationError> {
    let n = a.nrows();
    if a.ncols() != n {
        return Err(AllocationError::InvalidData);
    }
    let mut l = Array2::<f64>::zeros((n, n));
    for j in 0..n {
        fill_column(a, &mut l, j)?;
    }
    Ok(l)
}

/// Fills column `j` of the Cholesky factor, given the columns before it.
fn fill_column(a: &Array2<f64>, l: &mut Array2<f64>, j: usize) -> Result<(), AllocationError> {
    let root = pivot_root(residual(a, l, j, j))?;
    l[[j, j]] = root;
    for i in (j + 1)..a.nrows() {
        l[[i, j]] = residual(a, l, i, j) / root;
    }
    Ok(())
}

/// `a[i][j]` less what the first `j` columns of `l` already account for.
fn residual(a: &Array2<f64>, l: &Array2<f64>, i: usize, j: usize) -> f64 {
    a[[i, j]] - row_dot(l, i, j, j)
}

/// The square root of a Cholesky pivot, which must be positive.
fn pivot_root(pivot: f64) -> Result<f64, AllocationError> {
    if pivot.is_finite() && pivot > 0.0 {
        Ok(pivot.sqrt())
    } else {
        Err(AllocationError::InvalidData)
    }
}

/// `sum(l[i][k] * l[j][k])` for `k < len`.
fn row_dot(l: &Array2<f64>, i: usize, j: usize, len: usize) -> f64 {
    (0..len).map(|k| l[[i, k]] * l[[j, k]]).sum()
}

/// Solves `a x = b` for symmetric positive-definite `a`, by Cholesky.
///
/// # Errors
///
/// As [`cholesky`], and [`AllocationError::InputMismatch`] if `b` does not
/// match `a`'s size.
pub fn solve_spd(a: &Array2<f64>, b: ArrayView1<'_, f64>) -> Result<Array1<f64>, AllocationError> {
    if b.len() != a.nrows() {
        return Err(AllocationError::InputMismatch);
    }
    let l = cholesky(a)?;
    Ok(back_substitute(&l, &forward_substitute(&l, b)))
}

/// Solves `L y = b` for lower-triangular `L`.
fn forward_substitute(l: &Array2<f64>, b: ArrayView1<'_, f64>) -> Array1<f64> {
    let mut y = Array1::<f64>::zeros(b.len());
    for i in 0..b.len() {
        let known: f64 = (0..i).map(|k| l[[i, k]] * y[k]).sum();
        y[i] = (b[i] - known) / l[[i, i]];
    }
    y
}

/// Solves `L^T x = y` for lower-triangular `L`.
fn back_substitute(l: &Array2<f64>, y: &Array1<f64>) -> Array1<f64> {
    let n = y.len();
    let mut x = Array1::<f64>::zeros(n);
    for i in (0..n).rev() {
        let known: f64 = ((i + 1)..n).map(|k| l[[k, i]] * x[k]).sum();
        x[i] = (y[i] - known) / l[[i, i]];
    }
    x
}

/// The eigenvalues of a symmetric matrix, ascending. nalgebra reduces it
/// to tridiagonal form and runs implicit QR: O(n^3) once, a few
/// milliseconds for hundreds of assets. The iterations are capped at
/// `30 n`, LAPACK's bound.
///
/// # Errors
///
/// [`AllocationError::InvalidData`] if `a` is not square or not finite,
/// or if the iteration does not converge.
pub fn symmetric_eigenvalues(a: &Array2<f64>) -> Result<Vec<f64>, AllocationError> {
    let n = a.nrows();
    if a.ncols() != n || a.iter().any(|x| !x.is_finite()) {
        return Err(AllocationError::InvalidData);
    }
    let matrix = nalgebra::DMatrix::from_fn(n, n, |i, j| a[[i, j]]);
    let mut values: Vec<f64> =
        nalgebra::linalg::SymmetricEigen::try_new(matrix, f64::EPSILON, 30 * n.max(1))
            .ok_or(AllocationError::InvalidData)?
            .eigenvalues
            .iter()
            .copied()
            .collect();
    values.sort_by(f64::total_cmp);
    Ok(values)
}

/// The 2-norm condition number of a symmetric positive-definite matrix:
/// its largest eigenvalue over its smallest. Infinite if the smallest is
/// not positive.
///
/// # Errors
///
/// As [`symmetric_eigenvalues`].
pub fn condition_number(a: &Array2<f64>) -> Result<f64, AllocationError> {
    let values = symmetric_eigenvalues(a)?;
    let (lo, hi) = (values.first().copied().unwrap_or(0.0), values.last().copied().unwrap_or(0.0));
    Ok(if lo > 0.0 { hi / lo } else { f64::INFINITY })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::array;

    #[test]
    fn cholesky_reproduces_the_matrix() {
        let a = array![[4.0, 2.0, 0.4], [2.0, 5.0, 1.0], [0.4, 1.0, 3.0]];
        let l = cholesky(&a).unwrap();
        let back = l.dot(&l.t());
        assert!(a.iter().zip(back.iter()).all(|(x, y)| (x - y).abs() < 1e-12));
    }

    #[test]
    fn cholesky_rejects_an_indefinite_matrix() {
        assert_eq!(cholesky(&array![[1.0, 2.0], [2.0, 1.0]]), Err(AllocationError::InvalidData));
    }

    #[test]
    fn solve_spd_solves() {
        let a = array![[4.0, 1.0], [1.0, 3.0]];
        let x = solve_spd(&a, array![1.0, 2.0].view()).unwrap();
        assert!((x[0] - 1.0 / 11.0).abs() < 1e-12 && (x[1] - 7.0 / 11.0).abs() < 1e-12);
    }

    #[test]
    fn eigenvalues_of_a_known_matrix() {
        // [[2, 1], [1, 2]] has eigenvalues 1 and 3.
        let values = symmetric_eigenvalues(&array![[2.0, 1.0], [1.0, 2.0]]).unwrap();
        assert!((values[0] - 1.0).abs() < 1e-12 && (values[1] - 3.0).abs() < 1e-12);
        assert!((condition_number(&array![[2.0, 1.0], [1.0, 2.0]]).unwrap() - 3.0).abs() < 1e-12);
    }

    #[test]
    fn eigenvalues_sum_to_the_trace() {
        let a = array![[4.0, 2.0, 0.4], [2.0, 5.0, 1.0], [0.4, 1.0, 3.0]];
        let values = symmetric_eigenvalues(&a).unwrap();
        assert!((values.iter().sum::<f64>() - 12.0).abs() < 1e-10);
        assert!(values.windows(2).all(|w| w[0] <= w[1]));
    }

    #[test]
    fn a_hundred_assets_take_milliseconds_with_exact_values() {
        // 0.5 I + 0.01 J: eigenvalue 0.5 ninety-nine times, and
        // 0.5 + 100 x 0.01 = 1.5 once.
        let n = 100;
        let a = Array2::from_shape_fn((n, n), |(i, j)| if i == j { 0.51 } else { 0.01 });
        let values = symmetric_eigenvalues(&a).unwrap();
        assert!(values[..n - 1].iter().all(|v| (v - 0.5).abs() < 1e-12), "{values:?}");
        assert!((values[n - 1] - 1.5).abs() < 1e-12);
        assert!((condition_number(&a).unwrap() - 3.0).abs() < 1e-9);
    }

    #[test]
    fn a_wide_shrunk_covariance_takes_milliseconds() {
        // 30 days of 100 noisy assets, shrunk by Ledoit-Wolf: the shape on
        // which the Jacobi sweeps this replaced never met their stopping
        // rule and ran all 100 sweeps (66 s for 500 assets).
        let mut state: u64 = 0x9e37_79b9_7f4a_7c15;
        let returns = Array2::from_shape_fn((30, 100), |_| {
            state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
            (f64::from((state >> 33) as u32) / f64::from(u32::MAX) - 0.5) * 0.02
        });
        let covariance = crate::covariance::CovarianceEstimator::LedoitWolf
            .estimate(&returns)
            .unwrap()
            .covariance;
        let start = std::time::Instant::now();
        let values = symmetric_eigenvalues(&covariance).unwrap();
        let took = start.elapsed();
        assert!(took < std::time::Duration::from_secs(1), "took {took:?}");
        let trace: f64 = (0..100).map(|i| covariance[[i, i]]).sum();
        assert!((values.iter().sum::<f64>() - trace).abs() < 1e-12 * trace.max(1.0));
        assert!(values[0] > 0.0 && values.windows(2).all(|w| w[0] <= w[1]));
    }
}
