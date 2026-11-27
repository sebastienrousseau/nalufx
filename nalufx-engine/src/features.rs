//! Feature matrices for clustering.

use crate::horizon::MAX_ABS_DAILY_RETURN;
use crate::{
    check_empty_inputs, check_input_lengths, check_invalid_data, check_outliers,
    fill_feature_matrix, normalize_features, AllocationError,
};
use ndarray::Array2;

/// Extracts features from the input data for clustering.
///
/// The four series become the four columns of a feature matrix, each
/// normalised to zero mean and unit variance.
///
/// # Arguments
///
/// * `daily_returns` - A slice of daily returns.
/// * `cash_flows` - A slice of cash flows.
/// * `market_indices` - A slice of market indices.
/// * `fund_characteristics` - A slice of fund characteristics.
///
/// # Returns
///
/// A feature matrix (`Array2<f64>`) with one row per observation.
///
/// # Errors
///
/// Returns an error if the slices differ in length, are empty, hold NaN or
/// infinite values, or hold outliers.
///
/// # Examples
///
/// ```
/// use nalufx_engine::extract_features;
/// use nalufx_engine::AllocationError;
///
/// let daily_returns = vec![0.01, 0.02, -0.01];
/// let cash_flows = vec![1000.0, 1020.0, 1010.0];
/// let market_indices = vec![1.0, 1.01, 1.02];
/// let fund_characteristics = vec![0.5, 0.6, 0.7];
/// let features = extract_features!(&daily_returns, &cash_flows, &market_indices, &fund_characteristics).unwrap();
/// assert_eq!(features.shape(), &[3, 4]);
/// # Ok::<(), AllocationError>(())
/// ```
pub fn extract_features(
    daily_returns: &[f64],
    cash_flows: &[f64],
    market_indices: &[f64],
    fund_characteristics: &[f64],
) -> Result<Array2<f64>, AllocationError> {
    check_input_lengths!(daily_returns, cash_flows, market_indices, fund_characteristics)?;
    check_empty_inputs!(daily_returns, cash_flows, market_indices, fund_characteristics)?;
    check_invalid_data!(daily_returns, cash_flows)?;
    check_outliers!(MAX_ABS_DAILY_RETURN, daily_returns)?;
    check_outliers!(1_000_000.0, cash_flows)?;

    let n = daily_returns.len();
    let mut features = Array2::<f64>::zeros((n, 4));
    fill_feature_matrix!(
        features,
        n,
        daily_returns,
        cash_flows,
        market_indices,
        fund_characteristics
    );
    normalize_features!(features);
    Ok(features)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_column_is_standardised() {
        let f = extract_features(
            &[0.01, 0.02, 0.03],
            &[100.0, 200.0, 300.0],
            &[1.0, 1.0, 1.0],
            &[0.5, 0.6, 0.7],
        )
        .unwrap();
        assert_eq!(f.shape(), &[3, 4]);
        for col in [0, 1, 3] {
            let c = f.column(col);
            assert!(c.sum().abs() < 1e-12, "column {col} mean");
            assert!((c.mapv(|x| x * x).sum() / 3.0 - 1.0).abs() < 1e-12, "column {col} variance");
        }
        // A constant column is centred, not divided into NaN.
        assert!(f.column(2).iter().all(|x| *x == 0.0));
    }

    #[test]
    fn unusable_input_is_a_typed_error() {
        assert_eq!(
            extract_features(&[0.01], &[1.0, 2.0], &[1.0], &[1.0]),
            Err(AllocationError::InputMismatch)
        );
        assert_eq!(extract_features(&[], &[], &[], &[]), Err(AllocationError::EmptyInput));
        assert_eq!(
            extract_features(&[f64::NAN], &[1.0], &[1.0], &[1.0]),
            Err(AllocationError::InvalidData)
        );
        assert_eq!(
            extract_features(&[2.0], &[1.0], &[1.0], &[1.0]),
            Err(AllocationError::OutlierData)
        );
        assert_eq!(
            extract_features(&[0.1], &[2e6], &[1.0], &[1.0]),
            Err(AllocationError::OutlierData)
        );
    }
}
