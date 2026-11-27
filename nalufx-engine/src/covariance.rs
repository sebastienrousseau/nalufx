//! Expected returns and covariance from a returns matrix.
//!
//! Sample covariance is unbiased but unstable when there are fewer than
//! about ten observations per asset: its smallest eigenvalues collapse
//! towards zero and an optimiser loads onto them. Ledoit-Wolf shrinkage
//! pulls it towards a scaled identity by the amount that minimises the
//! expected squared error, and is the default.

use crate::AllocationError;
use ndarray::{Array1, Array2, Axis};
use serde::{Deserialize, Serialize};

/// How the covariance matrix is estimated.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CovarianceEstimator {
    /// The unbiased sample covariance, normalised by `T - 1`.
    Sample,
    /// Ledoit-Wolf (2004) shrinkage of the sample covariance towards
    /// `mu * I`, where `mu` is the average variance.
    #[default]
    LedoitWolf,
}

/// The estimated moments of a set of assets.
#[derive(Clone, Debug, PartialEq)]
pub struct Moments {
    /// The mean return of each asset, per period.
    pub mean: Array1<f64>,
    /// The covariance of the returns, per period.
    pub covariance: Array2<f64>,
    /// The Ledoit-Wolf shrinkage intensity, in `[0, 1]`; `None` for the
    /// sample estimator.
    pub shrinkage: Option<f64>,
}

impl CovarianceEstimator {
    /// Estimates the moments of a returns matrix.
    ///
    /// # Arguments
    ///
    /// * `returns` - One row per period, one column per asset.
    ///
    /// # Errors
    ///
    /// - [`AllocationError::EmptyInput`] if there are no assets.
    /// - [`AllocationError::InvalidParameter`] with fewer than two periods.
    /// - [`AllocationError::InvalidData`] if a return is NaN or infinite.
    ///
    /// # Examples
    ///
    /// ```
    /// use nalufx_engine::covariance::CovarianceEstimator;
    /// use ndarray::array;
    ///
    /// let returns = array![[0.01, 0.02], [0.03, -0.01], [-0.02, 0.01], [0.00, 0.02]];
    /// let sample = CovarianceEstimator::Sample.estimate(&returns).unwrap();
    /// assert!((sample.mean[0] - 0.005).abs() < 1e-15);
    /// let shrunk = CovarianceEstimator::LedoitWolf.estimate(&returns).unwrap();
    /// assert!((0.0..=1.0).contains(&shrunk.shrinkage.unwrap()));
    /// ```
    pub fn estimate(self, returns: &Array2<f64>) -> Result<Moments, AllocationError> {
        let (t, n) = returns.dim();
        if n == 0 {
            return Err(AllocationError::EmptyInput);
        }
        if t < 2 {
            return Err(AllocationError::InvalidParameter(
                "a covariance needs at least two periods".to_string(),
            ));
        }
        if returns.iter().any(|x| !x.is_finite()) {
            return Err(AllocationError::InvalidData);
        }
        let mean = returns.mean_axis(Axis(0)).ok_or(AllocationError::EmptyInput)?;
        let centred = returns - &mean;
        let cross = centred.t().dot(&centred);
        match self {
            Self::Sample => {
                let covariance = cross / (t as f64 - 1.0);
                Ok(Moments { mean, covariance, shrinkage: None })
            },
            Self::LedoitWolf => {
                let (covariance, shrinkage) = ledoit_wolf(&centred, &cross);
                Ok(Moments { mean, covariance, shrinkage: Some(shrinkage) })
            },
        }
    }
}

/// The Ledoit-Wolf estimate and its shrinkage intensity, as scikit-learn
/// computes it: the target is `mu * I` and the sample covariance is
/// normalised by `T`.
fn ledoit_wolf(centred: &Array2<f64>, cross: &Array2<f64>) -> (Array2<f64>, f64) {
    let sample = cross / centred.nrows() as f64;
    let mu = sample.diag().mean().unwrap_or(0.0);
    let shrinkage = intensity(centred, cross, mu);
    let mut shrunk = sample * (1.0 - shrinkage);
    shrunk.diag_mut().mapv_inplace(|v| shrinkage.mul_add(mu, v));
    (shrunk, shrinkage)
}

/// The shrinkage intensity: the sampling variance of the covariance's
/// entries over their squared distance from the target, capped at 1.
fn intensity(centred: &Array2<f64>, cross: &Array2<f64>, mu: f64) -> f64 {
    let delta = target_distance(cross, centred.nrows() as f64, mu);
    let beta = sampling_variance(centred, cross).min(delta);
    if beta <= 0.0 || delta <= 0.0 {
        0.0
    } else {
        beta / delta
    }
}

/// The estimated variance of the sample covariance's entries
/// (scikit-learn's `beta`).
fn sampling_variance(centred: &Array2<f64>, cross: &Array2<f64>) -> f64 {
    let (t, n) = centred.dim();
    let t = t as f64;
    let squared = centred.mapv(|x| x * x);
    let fourth = squared.t().dot(&squared).sum() / t;
    (fourth - frobenius_squared(cross) / (t * t)) / (n as f64 * t)
}

/// The squared Frobenius distance from the `T`-normalised sample
/// covariance to `mu * I`, per asset (scikit-learn's `delta`).
fn target_distance(cross: &Array2<f64>, t: f64, mu: f64) -> f64 {
    let n = cross.nrows() as f64;
    let trace = cross.diag().sum() / t;
    (frobenius_squared(cross) / (t * t) - 2.0 * mu * trace + n * mu * mu) / n
}

/// The sum of the squared entries.
fn frobenius_squared(m: &Array2<f64>) -> f64 {
    m.iter().map(|x| x * x).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::array;

    fn returns() -> Array2<f64> {
        array![
            [0.010, 0.020, -0.004],
            [0.030, -0.010, 0.006],
            [-0.020, 0.010, 0.001],
            [0.000, 0.020, -0.007],
            [0.015, -0.005, 0.002],
        ]
    }

    #[test]
    fn sample_covariance_is_unbiased() {
        let m = CovarianceEstimator::Sample.estimate(&returns()).unwrap();
        // Column 0: mean 0.007, deviations 0.003, 0.023, -0.027, -0.007, 0.008.
        let var0 =
            [0.003f64, 0.023, -0.027, -0.007, 0.008].iter().map(|d| d * d).sum::<f64>() / 4.0;
        assert!((m.covariance[[0, 0]] - var0).abs() < 1e-15);
        assert_eq!(m.covariance[[0, 1]], m.covariance[[1, 0]]);
        assert_eq!(m.shrinkage, None);
    }

    #[test]
    fn ledoit_wolf_keeps_the_trace_and_shrinks_off_diagonals() {
        let m = CovarianceEstimator::LedoitWolf.estimate(&returns()).unwrap();
        let s = m.shrinkage.unwrap();
        assert!(s > 0.0 && s <= 1.0, "shrinkage {s}");
        // The target has the same trace as the T-normalised sample.
        let sample_t =
            CovarianceEstimator::Sample.estimate(&returns()).unwrap().covariance * (4.0 / 5.0);
        assert!((m.covariance.diag().sum() - sample_t.diag().sum()).abs() < 1e-15);
        assert!((m.covariance[[0, 1]] - (1.0 - s) * sample_t[[0, 1]]).abs() < 1e-15);
    }

    #[test]
    fn ledoit_wolf_matches_the_reference_formula_by_hand() {
        // Two assets, three periods: worked through the formula by hand.
        // centred = [[1, 0], [-1, 1], [0, -1]] (means 0 and 0)
        let r = array![[1.0, 0.0], [-1.0, 1.0], [0.0, -1.0]];
        let m = CovarianceEstimator::LedoitWolf.estimate(&r).unwrap();
        // sample = X'X/3 = [[2, -1], [-1, 2]]/3; mu = 2/3.
        // beta_sum = sum((X^2)'X^2) = [[2, 1], [1, 2]] summed = 6;
        // delta_sum = sum((X'X)^2)/9 = (4 + 1 + 1 + 4)/9 = 10/9.
        // beta = (6/3 - 10/9)/(2*3) = 4/27; delta = (10/9 - 2*(2/3)*(4/3) + 2*(4/9))/2 = 1/9.
        // beta = min(4/27, 1/9) = 1/9, so the shrinkage is 1: the target.
        assert!((m.shrinkage.unwrap() - 1.0).abs() < 1e-12);
        assert!((m.covariance[[0, 0]] - 2.0 / 3.0).abs() < 1e-12);
        assert!(m.covariance[[0, 1]].abs() < 1e-12);
    }

    #[test]
    fn rejects_unusable_input() {
        assert_eq!(
            CovarianceEstimator::Sample.estimate(&Array2::zeros((5, 0))),
            Err(AllocationError::EmptyInput)
        );
        assert!(matches!(
            CovarianceEstimator::Sample.estimate(&Array2::zeros((1, 3))),
            Err(AllocationError::InvalidParameter(_))
        ));
        let mut bad = returns();
        bad[[2, 1]] = f64::NAN;
        assert_eq!(
            CovarianceEstimator::LedoitWolf.estimate(&bad),
            Err(AllocationError::InvalidData)
        );
    }
}
