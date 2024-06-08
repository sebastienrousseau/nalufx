/// Checks if the input slices have the same length.
///
/// This macro accepts multiple slices and verifies that all slices have the same length. If any
/// two slices have different lengths, it returns an `AllocationError::InputMismatch` error.
///
/// # Arguments
///
/// * `$input` - A variadic list of input slices to check for length equality.
///
/// # Example
///
/// ```
/// use nalufx::check_input_lengths;
/// use nalufx::errors::AllocationError;
///
/// let daily_returns = vec![0.01, 0.02, -0.01];
/// let cash_flows = vec![1000.0, 1020.0, 1010.0];
/// check_input_lengths!(daily_returns, cash_flows)?;
/// # Ok::<(), AllocationError>(())
/// ```
#[macro_export]
macro_rules! check_input_lengths {
    ($($input:expr),+ $(,)?) => {{
        let lengths = vec![$($input.len()),+];
        if lengths.windows(2).any(|w| w[0] != w[1]) {
            return Err(AllocationError::InputMismatch);
        }
        Ok::<(), AllocationError>(())
    }};
}

/// Checks if the input slices are empty.
///
/// This macro accepts multiple slices and verifies that none of the slices are empty. If any
/// slice is empty, it returns an `AllocationError::EmptyInput` error.
///
/// # Arguments
///
/// * `$input` - A variadic list of input slices to check for emptiness.
///
/// # Example
///
/// ```
/// use nalufx::check_empty_inputs;
/// use nalufx::errors::AllocationError;
///
/// let daily_returns = vec![0.01, 0.02, -0.01];
/// let cash_flows = vec![1000.0, 1020.0, 1010.0];
/// check_empty_inputs!(daily_returns, cash_flows)?;
/// # Ok::<(), AllocationError>(())
/// ```
#[macro_export]
macro_rules! check_empty_inputs {
    ($($input:expr),+ $(,)?) => {{
        if $($input.is_empty())||+ {
            return Err(AllocationError::EmptyInput);
        }
        Ok::<(), AllocationError>(())
    }};
}

/// Checks if the input slices contain invalid data (NaN or infinite values).
///
/// This macro accepts multiple slices and verifies that none of the slices contain NaN or
/// infinite values. If any slice contains invalid data, it returns an `AllocationError::InvalidData` error.
///
/// # Arguments
///
/// * `$input` - A variadic list of input slices to check for invalid data.
///
/// # Example
///
/// ```
/// use nalufx::check_invalid_data;
/// use nalufx::errors::AllocationError;
///
/// let daily_returns: Vec<f64> = vec![0.01, 0.02, -0.01];
/// let cash_flows: Vec<f64> = vec![1000.0, 1020.0, 1010.0];
/// check_invalid_data!(daily_returns, cash_flows)?;
/// # Ok::<(), AllocationError>(())
/// ```
#[macro_export]
macro_rules! check_invalid_data {
    ($($input:expr),+ $(,)?) => {{
        if $($input.iter().any(|&x| x.is_nan() || x.is_infinite()))||+ {
            return Err(AllocationError::InvalidData);
        }
        Ok::<(), AllocationError>(())
    }};
}

/// Checks if the input slices contain outlier values beyond the specified threshold.
///
/// This macro accepts multiple slices and a threshold value. It verifies that none of the slices
/// contain values beyond the specified threshold. If any slice contains outlier data, it returns
/// an `AllocationError::OutlierData` error.
///
/// # Arguments
///
/// * `$threshold` - The threshold value to check for outliers.
/// * `$input` - A variadic list of input slices to check for outlier data.
///
/// # Example
///
/// ```
/// use nalufx::check_outliers;
/// use nalufx::errors::AllocationError;
///
/// let daily_returns: Vec<f64> = vec![0.01, 0.02, -0.01];
/// let cash_flows: Vec<f64> = vec![1000.0, 1020.0, 1010.0];
/// check_outliers!(1.0, daily_returns)?;
/// check_outliers!(1_000_000.0, cash_flows)?;
/// # Ok::<(), AllocationError>(())
/// ```
#[macro_export]
macro_rules! check_outliers {
    ($threshold:expr, $($input:expr),+ $(,)?) => {{
        if $($input.iter().any(|&x| x.abs() > $threshold))||+ {
            return Err(AllocationError::OutlierData);
        }
        Ok::<(), AllocationError>(())
    }};
}

/// Handles the result of an operation and maps errors to a specified error type.
///
/// This macro accepts a result and an error type. If the result is an error, it maps the error
/// to the specified error type and returns it. If the result is Ok, it returns the value.
///
/// # Arguments
///
/// * `$result` - The result to handle.
/// * `$err_type` - The error type to map to in case of an error.
///
/// # Example
///
/// ```
/// use nalufx::handle_result;
/// use nalufx::utils::calculations::forecast_time_series;
/// use nalufx::errors::AllocationError;
///
/// let data: Vec<f64> = vec![0.01, 0.02, 0.03, 0.04, 0.05, 0.06, 0.07, 0.08, 0.09, 0.1];
/// let num_days: usize = 3;
/// let forecast_result: Result<Vec<f64>, String> = forecast_time_series(&data, num_days);
/// let forecasted_values = handle_result!(forecast_result, ForecastingError)?;
/// # Ok::<(), AllocationError>(())
/// ```
#[macro_export]
macro_rules! handle_result {
    ($result:expr, $err_type:ident) => {
        match $result {
            Ok(val) => Ok(val),
            Err(err) => Err(AllocationError::$err_type(err.to_string())),
        }
    };
}

/// Fills the feature matrix with values from the input slices.
///
/// This macro iterates over the input slices and fills the corresponding columns of the feature matrix.
///
/// # Arguments
///
/// * `$features` - The feature matrix to be filled.
/// * `$n` - The number of rows in the feature matrix.
/// * `$daily_returns` - A slice of daily returns.
/// * `$cash_flows` - A slice of cash flows.
/// * `$market_indices` - A slice of market indices.
/// * `$fund_characteristics` - A slice of fund characteristics.
///
/// # Example
///
/// ```
/// use nalufx::fill_feature_matrix;
/// use nalufx::errors::AllocationError;
/// use ndarray::Array2;
///
/// let daily_returns: Vec<f64> = vec![0.01, 0.02, -0.01];
/// let cash_flows: Vec<f64> = vec![1000.0, 1020.0, 1010.0];
/// let market_indices: Vec<f64> = vec![1.0, 1.01, 1.02];
/// let fund_characteristics: Vec<f64> = vec![0.5, 0.6, 0.7];
/// let n = daily_returns.len();
/// let mut features = Array2::<f64>::zeros((n, 4));
/// fill_feature_matrix!(features, n, daily_returns, cash_flows, market_indices, fund_characteristics);
/// # Ok::<(), AllocationError>(())
/// ```
#[macro_export]
macro_rules! fill_feature_matrix {
    ($features:expr, $n:expr, $daily_returns:expr, $cash_flows:expr, $market_indices:expr, $fund_characteristics:expr) => {{
        for i in 0..$n {
            $features[[i, 0]] = $daily_returns[i];
            $features[[i, 1]] = $cash_flows[i];
            $features[[i, 2]] = $market_indices[i];
            $features[[i, 3]] = $fund_characteristics[i];
        }
    }};
}

/// Normalizes the features by subtracting the mean and dividing by the standard deviation.
///
/// This macro calculates the mean and standard deviation of the features along the specified axis,
/// then normalizes the feature matrix.
///
/// # Arguments
///
/// * `$features` - The feature matrix to be normalized.
///
/// # Example
///
/// ```
/// use nalufx::normalize_features;
/// use nalufx::fill_feature_matrix;
/// use nalufx::errors::AllocationError;
/// use ndarray::Array2;
/// use ndarray::Axis;
///
/// let daily_returns: Vec<f64> = vec![0.01, 0.02, -0.01];
/// let cash_flows: Vec<f64> = vec![1000.0, 1020.0, 1010.0];
/// let market_indices: Vec<f64> = vec![1.0, 1.01, 1.02];
/// let fund_characteristics: Vec<f64> = vec![0.5, 0.6, 0.7];
/// let n = daily_returns.len();
/// let mut features = Array2::<f64>::zeros((n, 4));
/// fill_feature_matrix!(features, n, daily_returns, cash_flows, market_indices, fund_characteristics);
/// normalize_features!(features);
/// # Ok::<(), AllocationError>(())
/// ```
#[macro_export]
macro_rules! normalize_features {
    ($features:expr) => {{
        use ndarray::Axis;
        let mean = $features.mean_axis(Axis(0)).unwrap();
        let std_dev = $features.std_axis(Axis(0), 0.0);
        $features -= &mean;
        $features /= &std_dev;
    }};
}

/// Extracts and normalizes features from the input data.
///
/// This macro takes in the daily returns, cash flows, market indices, and fund characteristics,
/// and extracts them into a feature matrix. The feature matrix is then normalized by subtracting
/// the mean and dividing by the standard deviation along each column.
///
/// # Arguments
///
/// * `$features` - The mutable feature matrix to be filled and normalized.
/// * `$daily_returns` - A slice of daily returns.
/// * `$cash_flows` - A slice of cash flows.
/// * `$market_indices` - A slice of market indices.
/// * `$fund_characteristics` - A slice of fund characteristics.
///
/// # Returns
///
/// The normalized feature matrix.
///
/// # Example
///
/// ```
/// use nalufx::extract_features;
/// use nalufx::normalize_features;
/// use nalufx::fill_feature_matrix;
/// use nalufx::errors::AllocationError;
/// use ndarray::Array2;
///
/// let daily_returns: Vec<f64> = vec![0.01, 0.02, -0.01];
/// let cash_flows: Vec<f64> = vec![1000.0, 1020.0, 1010.0];
/// let market_indices: Vec<f64> = vec![1.0, 1.01, 1.02];
/// let fund_characteristics: Vec<f64> = vec![0.5, 0.6, 0.7];
/// let mut features = Array2::<f64>::zeros((daily_returns.len(), 4));
/// let normalized_features = extract_features!(daily_returns, cash_flows, market_indices, fund_characteristics)?;
/// # Ok::<(), AllocationError>(())
/// ```
#[macro_export]
macro_rules! extract_features {
    ($daily_returns:expr, $cash_flows:expr, $market_indices:expr, $fund_characteristics:expr) => {{
        let n = $daily_returns.len();
        let mut features = ndarray::Array2::<f64>::zeros((n, 4));
        $crate::fill_feature_matrix!(
            features,
            n,
            $daily_returns,
            $cash_flows,
            $market_indices,
            $fund_characteristics
        );
        $crate::normalize_features!(features);
        Ok::<_, $crate::errors::AllocationError>(features)
    }};
}
