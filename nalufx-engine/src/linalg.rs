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

/// The eigenvalues of a symmetric matrix, ascending, by cyclic Jacobi
/// rotations.
///
/// # Errors
///
/// [`AllocationError::InvalidData`] if `a` is not square or not finite.
pub fn symmetric_eigenvalues(a: &Array2<f64>) -> Result<Vec<f64>, AllocationError> {
    /// Sweeps before giving up; Jacobi converges quadratically, and a
    /// 50-asset matrix needs well under 15.
    const MAX_SWEEPS: usize = 100;
    let n = a.nrows();
    if a.ncols() != n || a.iter().any(|x| !x.is_finite()) {
        return Err(AllocationError::InvalidData);
    }
    let mut m = a.clone();
    let scale = m.iter().map(|x| x * x).sum::<f64>().sqrt().max(f64::MIN_POSITIVE);
    for _ in 0..MAX_SWEEPS {
        if off_diagonal_norm(&m) <= f64::EPSILON * scale {
            break;
        }
        for p in 0..n {
            for q in (p + 1)..n {
                rotate(&mut m, p, q);
            }
        }
    }
    let mut values: Vec<f64> = (0..n).map(|i| m[[i, i]]).collect();
    values.sort_by(f64::total_cmp);
    Ok(values)
}

/// The Frobenius norm of the off-diagonal part.
fn off_diagonal_norm(m: &Array2<f64>) -> f64 {
    let n = m.nrows();
    (0..n)
        .flat_map(|i| (0..n).filter(move |&j| j != i).map(move |j| (i, j)))
        .map(|(i, j)| m[[i, j]].powi(2))
        .sum::<f64>()
        .sqrt()
}

/// One Jacobi rotation zeroing `m[p][q]` (and `m[q][p]`).
fn rotate(m: &mut Array2<f64>, p: usize, q: usize) {
    if m[[p, q]] == 0.0 {
        return;
    }
    let cs = rotation(m[[p, p]], m[[q, q]], m[[p, q]]);
    // Columns p and q, then rows p and q: m <- J' m J.
    turn_lines(m, p, q, cs, column_entry);
    turn_lines(m, p, q, cs, row_entry);
}

/// Entry `k` of column `line`.
const fn column_entry(k: usize, line: usize) -> [usize; 2] {
    [k, line]
}

/// Entry `k` of row `line`.
const fn row_entry(k: usize, line: usize) -> [usize; 2] {
    [line, k]
}

/// Turns lines `p` and `q` of `m` by the rotation `cs`; `at(k, line)`
/// names the entry `k` of a line (a column or a row).
fn turn_lines(
    m: &mut Array2<f64>,
    p: usize,
    q: usize,
    cs: (f64, f64),
    at: impl Fn(usize, usize) -> [usize; 2],
) {
    for k in 0..m.nrows() {
        let (x, y) = turn(cs.0, cs.1, m[at(k, p)], m[at(k, q)]);
        m[at(k, p)] = x;
        m[at(k, q)] = y;
    }
}

/// The cosine and sine of the rotation that zeroes the off-diagonal
/// entry `apq` between diagonal entries `app` and `aqq`.
fn rotation(app: f64, aqq: f64, apq: f64) -> (f64, f64) {
    let theta = (aqq - app) / (2.0 * apq);
    // `signum(0.0)` is 1, so a zero theta rotates by 45 degrees.
    let t = theta.signum() / (theta.abs() + theta.mul_add(theta, 1.0).sqrt());
    let c = 1.0 / t.mul_add(t, 1.0).sqrt();
    (c, t * c)
}

/// The pair `(x, y)` turned by the rotation `(c, s)`.
fn turn(c: f64, s: f64, x: f64, y: f64) -> (f64, f64) {
    (c.mul_add(x, -s * y), s.mul_add(x, c * y))
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
}
