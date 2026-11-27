//! Screening assets by a score before allocating, such as an ESG rating.
//!
//! NaluFX has no source of ESG or other ratings of its own: the scores
//! are the caller's, and the screen only applies a threshold to them.

use crate::AllocationError;
use ndarray::{Array2, Axis};

/// The assets whose score is at least `min_score`, and their columns of
/// `returns`, in their original order.
///
/// # Arguments
///
/// * `assets` - The asset names, one per column of `returns`.
/// * `scores` - One score per asset, on whatever scale the caller uses.
/// * `returns` - One row per period, one column per asset.
/// * `min_score` - The lowest score an asset may have and stay in.
///
/// # Errors
///
/// - [`AllocationError::InputMismatch`] if the three disagree on the
///   number of assets.
/// - [`AllocationError::InvalidData`] if a score or the threshold is NaN.
/// - [`AllocationError::EmptyInput`] if no asset passes.
///
/// # Examples
///
/// ```
/// use nalufx_engine::screening::screen;
/// use ndarray::array;
///
/// let assets = ["A".to_string(), "B".to_string(), "C".to_string()];
/// let returns = array![[0.01, 0.02, 0.03], [0.00, -0.01, 0.02]];
/// let (kept, kept_returns) = screen(&assets, &[7.5, 4.0, 8.1], &returns, 6.0).unwrap();
/// assert_eq!(kept, vec!["A", "C"]);
/// assert_eq!(kept_returns.column(1).to_vec(), vec![0.03, 0.02]);
/// ```
pub fn screen(
    assets: &[String],
    scores: &[f64],
    returns: &Array2<f64>,
    min_score: f64,
) -> Result<(Vec<String>, Array2<f64>), AllocationError> {
    if assets.len() != scores.len() || assets.len() != returns.ncols() {
        return Err(AllocationError::InputMismatch);
    }
    if min_score.is_nan() || scores.iter().any(|s| s.is_nan()) {
        return Err(AllocationError::InvalidData);
    }
    let keep: Vec<usize> = (0..assets.len()).filter(|&i| scores[i] >= min_score).collect();
    if keep.is_empty() {
        return Err(AllocationError::EmptyInput);
    }
    let names = keep.iter().map(|&i| assets[i].clone()).collect();
    Ok((names, returns.select(Axis(1), &keep)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::array;

    fn assets() -> Vec<String> {
        vec!["A".into(), "B".into()]
    }

    #[test]
    fn nothing_passing_is_an_error() {
        let r = array![[0.0, 0.0]];
        assert_eq!(screen(&assets(), &[1.0, 2.0], &r, 3.0), Err(AllocationError::EmptyInput));
    }

    #[test]
    fn mismatched_lengths_are_an_error() {
        let r = array![[0.0, 0.0]];
        assert_eq!(screen(&assets(), &[1.0], &r, 0.0), Err(AllocationError::InputMismatch));
    }

    #[test]
    fn a_nan_score_is_an_error() {
        let r = array![[0.0, 0.0]];
        assert_eq!(screen(&assets(), &[1.0, f64::NAN], &r, 0.0), Err(AllocationError::InvalidData));
    }
}
