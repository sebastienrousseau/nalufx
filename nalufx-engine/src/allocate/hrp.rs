//! Hierarchical risk parity (López de Prado, 2016).
//!
//! No matrix is inverted, so it stays stable on a near-singular
//! covariance where minimum variance would load onto noise.

use super::{project, Allocator, Constraints, Solution};
use crate::covariance::Moments;
use crate::AllocationError;
use ndarray::Array2;

/// Hierarchical risk parity:
///
/// 1. Assets are clustered by single linkage on the Euclidean distance
///    between their correlation-distance profiles, where the correlation
///    distance is `sqrt((1 - rho) / 2)`.
/// 2. The covariance is reordered so similar assets sit together (the
///    dendrogram's leaf order).
/// 3. Recursive bisection splits that order in half again and again,
///    giving each half a share inversely proportional to its variance
///    under inverse-variance weights.
///
/// Ties are broken towards the lowest index, so the result is
/// deterministic. Weights that break the bounds are projected onto them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Hrp;

impl Allocator for Hrp {
    fn name(&self) -> &'static str {
        "hrp"
    }

    fn params(&self) -> serde_json::Value {
        serde_json::json!({ "linkage": "single", "distance": "correlation" })
    }

    fn solve(
        &self,
        moments: &Moments,
        constraints: &Constraints,
    ) -> Result<Solution, AllocationError> {
        let s = &moments.covariance;
        let n = s.nrows();
        if n == 0 {
            return Err(AllocationError::EmptyInput);
        }
        if s.diag().iter().any(|v| !(v.is_finite() && *v > 0.0)) {
            return Err(AllocationError::ZeroVolatility);
        }
        let order = leaf_order(&profile_distances(s), n);
        let raw = bisect(s, &order);
        let projected =
            raw.iter().any(|w| *w > constraints.max_weight || *w < constraints.min_weight);
        let weights = if projected { project(&raw, constraints)? } else { raw };
        Ok(Solution { weights, solver_status: "closed_form".to_string(), iterations: 0, projected })
    }
}

/// Euclidean distances between the columns of the correlation-distance
/// matrix.
fn profile_distances(s: &Array2<f64>) -> Array2<f64> {
    let n = s.nrows();
    let sd: Vec<f64> = s.diag().iter().map(|v| v.sqrt()).collect();
    let d = Array2::from_shape_fn((n, n), |(i, j)| {
        let rho = (s[[i, j]] / (sd[i] * sd[j])).clamp(-1.0, 1.0);
        (0.5 * (1.0 - rho)).sqrt()
    });
    Array2::from_shape_fn((n, n), |(i, j)| {
        (0..n).map(|k| (d[[k, i]] - d[[k, j]]).powi(2)).sum::<f64>().sqrt()
    })
}

/// The leaf order of the single-linkage dendrogram over `n` items.
///
/// Each merge joins the two closest clusters (lowest indices first on a
/// tie) and places the one formed earlier on the left.
fn leaf_order(distances: &Array2<f64>, n: usize) -> Vec<usize> {
    let mut clusters: Vec<Vec<usize>> = (0..n).map(|i| vec![i]).collect();
    while clusters.len() > 1 {
        let (a, b) = closest_pair(distances, &clusters);
        let right = clusters.remove(b);
        clusters[a].extend(right);
    }
    clusters.pop().unwrap_or_default()
}

/// The two clusters (`a < b`) with the smallest single-linkage distance.
fn closest_pair(distances: &Array2<f64>, clusters: &[Vec<usize>]) -> (usize, usize) {
    let mut best = (0, 1, f64::INFINITY);
    for a in 0..clusters.len() {
        for b in (a + 1)..clusters.len() {
            let d = linkage(distances, &clusters[a], &clusters[b]);
            if d < best.2 {
                best = (a, b, d);
            }
        }
    }
    (best.0, best.1)
}

/// The single-linkage distance: the closest pair across two clusters.
fn linkage(distances: &Array2<f64>, a: &[usize], b: &[usize]) -> f64 {
    a.iter().flat_map(|&i| b.iter().map(move |&j| distances[[i, j]])).fold(f64::INFINITY, f64::min)
}

/// Recursive bisection over the ordered assets.
fn bisect(s: &Array2<f64>, order: &[usize]) -> Vec<f64> {
    let mut weights = vec![1.0; s.nrows()];
    let mut stack = vec![order.to_vec()];
    while let Some(cluster) = stack.pop() {
        if cluster.len() < 2 {
            continue;
        }
        let (left, right) = cluster.split_at(cluster.len() / 2);
        let alpha = left_share(s, left, right);
        scale(&mut weights, left, alpha);
        scale(&mut weights, right, 1.0 - alpha);
        stack.push(right.to_vec());
        stack.push(left.to_vec());
    }
    weights
}

/// The left half's share of its parent: inversely proportional to its
/// variance, `1 - v_left / (v_left + v_right)`.
fn left_share(s: &Array2<f64>, left: &[usize], right: &[usize]) -> f64 {
    let (vl, vr) = (cluster_variance(s, left), cluster_variance(s, right));
    1.0 - vl / (vl + vr)
}

/// Multiplies the members' weights by `factor`.
fn scale(weights: &mut [f64], members: &[usize], factor: f64) {
    for &i in members {
        weights[i] *= factor;
    }
}

/// The variance of a cluster under inverse-variance weights.
fn cluster_variance(s: &Array2<f64>, members: &[usize]) -> f64 {
    let inverse: Vec<f64> = members.iter().map(|&i| 1.0 / s[[i, i]]).collect();
    let total: f64 = inverse.iter().sum();
    let w: Vec<f64> = inverse.iter().map(|x| x / total).collect();
    members
        .iter()
        .zip(&w)
        .flat_map(|(&i, wi)| members.iter().zip(&w).map(move |(&j, wj)| wi * wj * s[[i, j]]))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::{array, Array1};

    fn moments(covariance: Array2<f64>) -> Moments {
        let n = covariance.nrows();
        Moments { mean: Array1::zeros(n), covariance, shrinkage: None }
    }

    /// Two uncorrelated assets: one split, shares inversely proportional
    /// to the variances (1 and 4 give 0.8 and 0.2).
    #[test]
    fn two_uncorrelated_assets_get_inverse_variance() {
        let w = Hrp
            .solve(&moments(array![[1.0, 0.0], [0.0, 4.0]]), &Constraints::default())
            .unwrap()
            .weights;
        assert!((w[0] - 0.8).abs() < 1e-12 && (w[1] - 0.2).abs() < 1e-12, "{w:?}");
    }

    /// Two tight pairs: the dendrogram keeps each pair together.
    #[test]
    fn correlated_pairs_are_ordered_together() {
        let s = array![
            [1.0, 0.0, 0.9, 0.0],
            [0.0, 1.0, 0.0, 0.9],
            [0.9, 0.0, 1.0, 0.0],
            [0.0, 0.9, 0.0, 1.0],
        ];
        let order = leaf_order(&profile_distances(&s), 4);
        let pos = |i| order.iter().position(|&x| x == i).unwrap();
        assert_eq!(pos(0).abs_diff(pos(2)), 1, "{order:?}");
        assert_eq!(pos(1).abs_diff(pos(3)), 1, "{order:?}");
        // Symmetric pairs split the budget equally.
        let w = Hrp.solve(&moments(s), &Constraints::default()).unwrap().weights;
        assert!(w.iter().all(|x| (x - 0.25).abs() < 1e-12), "{w:?}");
    }

    #[test]
    fn a_zero_variance_asset_is_an_error() {
        let s = array![[1.0, 0.0], [0.0, 0.0]];
        assert_eq!(
            Hrp.solve(&moments(s), &Constraints::default()),
            Err(AllocationError::ZeroVolatility)
        );
    }
}
