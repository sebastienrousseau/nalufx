//! Deterministic K-means clustering of feature rows.
//!
//! The same feature matrix always gives the same labels: centroids are
//! seeded by farthest-point selection instead of a random draw, and every
//! tie is broken towards the lowest row or cluster index.

use crate::errors::AllocationError;
use ndarray::{Array1, Array2, ArrayView1};

/// Number of clusters.
const N_CLUSTERS: usize = 2;

/// Upper bound on Lloyd iterations; the loop normally converges first.
const MAX_ITERATIONS: usize = 100;

/// Assigns each row of a feature matrix to one of two clusters.
///
/// Seeding is deterministic: the first centroid is the row nearest the
/// column means, and each further centroid is the row farthest from the
/// centroids chosen so far (the maximin rule). Lloyd iterations then
/// refine the centroids until no label changes.
///
/// # Arguments
///
/// * `features` - A feature matrix, one row per observation.
///
/// # Returns
///
/// One cluster label (`0` or `1`) per row.
///
/// # Errors
///
/// Returns [`AllocationError::ClusteringError`] if the matrix has no rows.
///
/// # Examples
///
/// ```
/// use nalufx::utils::clustering::perform_clustering;
/// use ndarray::Array2;
/// let features = Array2::from_shape_vec((3, 4), vec![0.0; 12]).unwrap();
/// let clusters = perform_clustering(&features).unwrap();
/// assert_eq!(clusters.len(), 3);
/// ```
pub fn perform_clustering(features: &Array2<f64>) -> Result<Vec<usize>, AllocationError> {
    let n_samples = features.nrows();
    if n_samples == 0 {
        return Err(AllocationError::ClusteringError(
            "cannot cluster an empty feature matrix".to_string(),
        ));
    }
    // With no more samples than clusters, each sample is its own cluster.
    if n_samples <= N_CLUSTERS {
        return Ok((0..n_samples).collect());
    }

    let mut centroids = seed_centroids(features);
    let mut labels = vec![0usize; n_samples];
    for _ in 0..MAX_ITERATIONS {
        let changed = assign(features, &centroids, &mut labels);
        update_centroids(features, &labels, &mut centroids);
        if !changed {
            break;
        }
    }
    Ok(labels)
}

/// Picks the initial centroids by the maximin rule.
fn seed_centroids(features: &Array2<f64>) -> Vec<Array1<f64>> {
    let mean =
        features.mean_axis(ndarray::Axis(0)).unwrap_or_else(|| Array1::zeros(features.ncols()));
    let first = argmin(features.rows().into_iter().map(|row| squared_distance(row, mean.view())));
    let mut centroids = vec![features.row(first).to_owned()];
    while centroids.len() < N_CLUSTERS {
        let next = argmax(features.rows().into_iter().map(|row| nearest(row, &centroids).1));
        centroids.push(features.row(next).to_owned());
    }
    centroids
}

/// Labels each row with its nearest centroid; returns whether any changed.
fn assign(features: &Array2<f64>, centroids: &[Array1<f64>], labels: &mut [usize]) -> bool {
    let mut changed = false;
    for (row, label) in features.rows().into_iter().zip(labels.iter_mut()) {
        let (k, _) = nearest(row, centroids);
        changed |= *label != k;
        *label = k;
    }
    changed
}

/// Moves each centroid to the mean of its members. A cluster with no
/// members keeps its previous centroid rather than becoming NaN.
fn update_centroids(features: &Array2<f64>, labels: &[usize], centroids: &mut [Array1<f64>]) {
    for (k, centroid) in centroids.iter_mut().enumerate() {
        let mut sum = Array1::<f64>::zeros(features.ncols());
        let mut count = 0usize;
        for (row, _) in features.rows().into_iter().zip(labels).filter(|(_, &l)| l == k) {
            sum += &row;
            count += 1;
        }
        if count > 0 {
            *centroid = sum / count as f64;
        }
    }
}

/// The index of the nearest centroid and its squared distance; the lowest
/// index wins a tie.
fn nearest(row: ArrayView1<'_, f64>, centroids: &[Array1<f64>]) -> (usize, f64) {
    let distances: Vec<f64> = centroids.iter().map(|c| squared_distance(row, c.view())).collect();
    let k = argmin(distances.iter().copied());
    (k, distances.get(k).copied().unwrap_or(f64::INFINITY))
}

/// The index of the smallest value; the first one wins a tie.
fn argmin(values: impl Iterator<Item = f64>) -> usize {
    values
        .enumerate()
        .fold((0, f64::INFINITY), |best, (i, v)| if v < best.1 { (i, v) } else { best })
        .0
}

/// The index of the largest value; the first one wins a tie.
fn argmax(values: impl Iterator<Item = f64>) -> usize {
    values
        .enumerate()
        .fold((0, f64::NEG_INFINITY), |best, (i, v)| if v > best.1 { (i, v) } else { best })
        .0
}

/// Squared Euclidean distance between two equal-length rows. The square
/// root is monotonic, so it does not change which centroid is nearest.
fn squared_distance(a: ArrayView1<'_, f64>, b: ArrayView1<'_, f64>) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum()
}

#[cfg(test)]
mod tests {
    use super::perform_clustering;
    use ndarray::Array2;

    /// Two well-separated groups must land in two different clusters,
    /// with every member of a group agreeing.
    #[test]
    fn separates_two_distinct_groups() {
        let features = Array2::from_shape_vec(
            (6, 2),
            vec![
                0.0, 0.0, 0.1, 0.1, 0.2, 0.0, // tight cluster near the origin
                50.0, 50.0, 50.1, 49.9, 49.9, 50.2, // and one far away
            ],
        )
        .unwrap();

        let clusters = perform_clustering(&features).unwrap();
        assert_eq!(clusters.len(), 6);

        let (low, high) = clusters.split_at(3);
        assert!(low.iter().all(|&c| c == low[0]), "near group split: {clusters:?}");
        assert!(high.iter().all(|&c| c == high[0]), "far group split: {clusters:?}");
        assert_ne!(low[0], high[0], "the two groups collapsed: {clusters:?}");
    }

    /// The same matrix gives the same labels on every call.
    #[test]
    fn is_deterministic() {
        let features = Array2::from_shape_fn((40, 3), |(i, j)| ((i * 7 + j * 3) % 11) as f64);
        let first = perform_clustering(&features).unwrap();
        for _ in 0..20 {
            assert_eq!(perform_clustering(&features).unwrap(), first);
        }
    }

    /// All-identical rows have no meaningful split; it must not divide by
    /// zero or emit an out-of-range label.
    #[test]
    fn handles_identical_rows() {
        let features = Array2::from_shape_vec((3, 4), vec![0.0; 12]).unwrap();
        assert_eq!(perform_clustering(&features).unwrap(), vec![0, 0, 0]);
    }

    /// Fewer samples than clusters short-circuits rather than trying to
    /// seed distinct centroids.
    #[test]
    fn handles_fewer_samples_than_clusters() {
        let features = Array2::from_shape_vec((2, 2), vec![1.0, 2.0, 3.0, 4.0]).unwrap();
        assert_eq!(perform_clustering(&features).unwrap(), vec![0, 1]);
    }

    /// An empty matrix is an error, not a panic.
    #[test]
    fn rejects_empty_input() {
        let features = Array2::<f64>::zeros((0, 4));
        assert!(perform_clustering(&features).is_err());
    }

    /// Labels are always valid cluster indices.
    #[test]
    fn labels_are_in_range() {
        let features =
            Array2::from_shape_vec((5, 2), vec![1.0, 1.0, 2.0, 2.0, 3.0, 3.0, 9.0, 9.0, 8.0, 8.0])
                .unwrap();
        let clusters = perform_clustering(&features).unwrap();
        assert_eq!(clusters, vec![0, 0, 0, 1, 1]);
    }
}
