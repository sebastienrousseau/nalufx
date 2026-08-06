use crate::errors::AllocationError;
use crate::{
    check_empty_inputs, check_input_lengths, check_invalid_data, check_outliers,
    fill_feature_matrix, handle_result, normalize_features,
};
use augurs_core::{Fit, Predict};
use augurs_ets::AutoETS;
use ndarray::prelude::*;
use std::cmp::Ordering;
// rand 0.9 renamed `thread_rng()` to `rng()` and moved `gen_range` to
// `RngExt::random_range`.
use rand::RngExt;

/// Calculates the optimal allocation based on daily returns and cash flows.
///
/// This function uses a combination of time series forecasting, sentiment analysis,
/// reinforcement learning, and clustering to calculate the optimal allocation for each day.
///
/// # Arguments
///
/// * `daily_returns` - A slice of daily returns.
/// * `cash_flows` - A slice of cash flows.
/// * `market_indices` - A slice of market indices.
/// * `fund_characteristics` - A slice of fund characteristics.
/// * `num_days` - The number of days to generate predictions for.
///
/// # Returns
///
/// A vector of optimal allocations for each day, or an error if input slices have different lengths.
///
/// # Errors
///
/// Returns an error if:
/// - The input slices have different lengths.
/// - The input slices are empty.
/// - An error occurs during the execution of the `perform_clustering` function.
///
/// # Examples
///
/// ```
/// use nalufx::utils::calculations::calculate_optimal_allocation;
///
/// let daily_returns = vec![0.01, 0.02, -0.01, 0.03, 0.01];
/// let cash_flows = vec![1000.0, 1020.0, 1010.0, 1030.0, 1025.0];
/// let market_indices = vec![1.0, 1.01, 1.02, 1.03, 1.04];
/// let fund_characteristics = vec![0.5, 0.6, 0.7, 0.8, 0.9];
/// let num_days = 3;
/// match calculate_optimal_allocation(&daily_returns, &cash_flows, &market_indices, &fund_characteristics, num_days) {
///     Ok(allocations) => println!("Allocations: {:?}", allocations),
///   Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn calculate_optimal_allocation(
    daily_returns: &[f64],
    cash_flows: &[f64],
    market_indices: &[f64],
    fund_characteristics: &[f64],
    num_days: usize,
) -> Result<Vec<f64>, AllocationError> {
    // Check input lengths
    check_input_lengths!(daily_returns, cash_flows, market_indices, fund_characteristics)?;

    // Check for empty inputs
    check_empty_inputs!(daily_returns, cash_flows, market_indices, fund_characteristics)?;

    // Check for invalid data
    check_invalid_data!(daily_returns, cash_flows)?;

    // Check for outliers
    check_outliers!(1.0, daily_returns)?;
    check_outliers!(1_000_000.0, cash_flows)?;

    // Feature Engineering
    let features =
        extract_features(daily_returns, cash_flows, market_indices, fund_characteristics)?;

    // Time Series Forecasting
    let forecasted_returns =
        handle_result!(forecast_time_series(daily_returns, num_days), ForecastingError)?;
    let forecasted_cash_flows =
        handle_result!(forecast_time_series(cash_flows, num_days), ForecastingError)?;

    // Sentiment Analysis
    let sentiment_scores = handle_result!(analyze_sentiment(num_days), SentimentAnalysisError)?;

    // Reinforcement Learning
    let optimal_actions =
        handle_result!(train_reinforcement_learning(num_days), ReinforcementLearningError)?;

    // Clustering
    let clusters = match perform_clustering(&features) {
        Ok(clusters) => clusters,
        Err(err) => {
            eprintln!("Error during clustering: {}", err);
            vec![0; num_days]
        },
    };

    // Calculate averages
    let avg_daily_return = daily_returns.iter().sum::<f64>() / daily_returns.len() as f64;
    let avg_cash_flow = cash_flows.iter().sum::<f64>() / cash_flows.len() as f64;

    // Initialize predictions vector
    let mut predictions = Vec::with_capacity(num_days);

    // Calculate predictions in one pass
    for day in 1..=num_days {
        let predicted_return = if day <= forecasted_returns.len() {
            forecasted_returns[day - 1]
        } else {
            avg_daily_return * day as f64
        };

        let predicted_cash_flow = if day <= forecasted_cash_flows.len() {
            forecasted_cash_flows[day - 1]
        } else {
            avg_cash_flow * day as f64
        };

        // Check if the day index is within the valid range
        if day <= sentiment_scores.len() && day <= optimal_actions.len() && day <= clusters.len() {
            let sentiment_score = sentiment_scores[day - 1];
            let optimal_action = optimal_actions[day - 1];
            let cluster = clusters[day - 1] as f64;

            // Incorporate sentiment score, optimal action, and cluster into the prediction
            let prediction = predicted_return
                * predicted_cash_flow
                * sentiment_score
                * optimal_action
                * (cluster + 1.0);
            predictions.push(prediction);
        } else {
            // If the day index is out of range, use default values
            let prediction = predicted_return * predicted_cash_flow;
            predictions.push(prediction);
        }
    }

    // Calculate total prediction to normalize the predictions
    let total_prediction: f64 = predictions.iter().sum();

    // Handle the case where total prediction is zero
    if total_prediction == 0.0 {
        return Ok(vec![0.0; num_days]);
    }

    // Normalize predictions to get the optimal allocations
    Ok(predictions.into_iter().map(|p| p / total_prediction).collect())
}

/// Extracts features from the input data for clustering.
///
/// This function takes slices of daily returns, cash flows, market indices, and fund characteristics,
/// and constructs a feature matrix for clustering. It normalizes the features before returning them.
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
/// A feature matrix (`Array2<f64>`) for clustering, or an error if input slices have different lengths.
///
/// # Errors
///
/// Returns an error if the input slices have different lengths.
///
/// # Examples
///
/// ```
/// use nalufx::extract_features;
/// use nalufx::errors::AllocationError;
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
    // Check if input slices have the same length
    check_input_lengths!(daily_returns, cash_flows, market_indices, fund_characteristics)?;

    // Check for empty inputs
    check_empty_inputs!(daily_returns, cash_flows, market_indices, fund_characteristics)?;

    // Check for invalid data
    check_invalid_data!(daily_returns, cash_flows)?;

    // Check for outliers
    check_outliers!(1.0, daily_returns)?;
    check_outliers!(1_000_000.0, cash_flows)?;

    let n = daily_returns.len();
    let mut features = Array2::<f64>::zeros((n, 4));

    // Fill the feature matrix
    fill_feature_matrix!(
        features,
        n,
        daily_returns,
        cash_flows,
        market_indices,
        fund_characteristics
    );

    // Normalize the features
    normalize_features!(features);

    Ok(features)
}

/// Forecasts future values of a time series using the AutoETS model.
///
/// This function takes a slice of historical data and forecasts future values
/// for the specified number of days using the AutoETS model.
///
/// # Arguments
///
/// * `data` - A slice of historical data.
/// * `num_days` - The number of days to forecast.
///
/// # Returns
///
/// A vector of forecasted values (`Vec<f64>`) for the specified number of days, or an error if forecasting fails.
///
/// # Errors
///
/// Returns an error if the AutoETS model fails to fit the data or generate forecasts.
///
/// # Examples
///
/// ```
/// use nalufx::utils::calculations::forecast_time_series;
///
/// let data = vec![100.0, 101.0, 102.0, 101.5];
/// let num_days = 3;
/// match forecast_time_series(&data, num_days) {
///     Ok(forecast) => println!("Forecast: {:?}", forecast),
///   Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn forecast_time_series(data: &[f64], num_days: usize) -> Result<Vec<f64>, String> {
    let search = AutoETS::new(1, "ZZN").map_err(|e| e.to_string())?;
    let model = search.fit(data).map_err(|e| e.to_string())?;
    // augurs 0.10 made `predict` fallible; it returned the Forecast
    // directly in 0.9.
    let forecast = model.predict(num_days, 0.95).map_err(|e| e.to_string())?;
    Ok(forecast.point)
}

/// Analyzes sentiment scores for a given number of days.
///
/// This function generates sentiment scores for the specified number of days.
/// The actual implementation should replace the placeholder logic.
///
/// # Arguments
///
/// * `num_days` - The number of days for which to generate sentiment scores.
///
/// # Returns
///
/// A vector of sentiment scores (`Vec<f64>`) for the specified number of days, or an error if sentiment analysis fails.
///
/// # Errors
///
/// Returns an error if the sentiment analysis fails.
///
/// # Examples
///
/// ```
/// use nalufx::utils::calculations::analyze_sentiment;
/// let num_days = 3;
/// let sentiment_scores = analyze_sentiment(num_days).unwrap();
/// assert_eq!(sentiment_scores.len(), num_days);
/// ```
pub fn analyze_sentiment(num_days: usize) -> Result<Vec<f64>, String> {
    // Call the sentiment analysis helper function
    let sentiment_scores = get_sentiment_scores(num_days)?;
    Ok(sentiment_scores)
}

/// Trains a reinforcement learning model to generate optimal actions for a given number of days.
///
/// This function generates optimal actions for the specified number of days using reinforcement learning.
/// The actual implementation should replace the placeholder logic.
///
/// # Arguments
///
/// * `num_days` - The number of days for which to generate optimal actions.
///
/// # Returns
///
/// A vector of optimal actions (`Vec<f64>`) for the specified number of days, or an error if reinforcement learning fails.
///
/// # Errors
///
/// Returns an error if the reinforcement learning process fails.
///
/// # Examples
///
/// ```
/// use nalufx::utils::calculations::train_reinforcement_learning;
/// let num_days = 3;
/// let optimal_actions = train_reinforcement_learning(num_days).unwrap();
/// assert_eq!(optimal_actions.len(), num_days);
/// ```
pub fn train_reinforcement_learning(num_days: usize) -> Result<Vec<f64>, String> {
    // Call the reinforcement learning helper function
    let optimal_actions = get_optimal_actions(num_days)?;
    Ok(optimal_actions)
}

/// Performs clustering on the feature matrix using K-means with hyperparameter tuning.
///
/// This function takes a feature matrix and performs K-means clustering to assign each data point to a cluster.
///
/// # Arguments
///
/// * `features` - A reference to the feature matrix (`Array2<f64>`).
///
/// # Returns
///
/// A vector of cluster assignments (`Vec<usize>`) for each data point, or an error if clustering fails.
///
/// # Errors
///
/// Returns an error if the K-means model fails to fit the data or generate cluster assignments.
///
/// # Examples
///
/// ```
/// use nalufx::utils::calculations::perform_clustering;
/// use ndarray::Array2;
/// let features = Array2::from_shape_vec((3, 4), vec![0.0; 12]).unwrap();
/// let clusters = perform_clustering(&features).unwrap();
/// assert_eq!(clusters.len(), 3);
/// ```
pub fn perform_clustering(features: &Array2<f64>) -> Result<Vec<usize>, AllocationError> {
    /// Number of clusters. Matches the previous linfa configuration.
    const N_CLUSTERS: usize = 2;
    /// Upper bound on Lloyd iterations; the loop normally converges first.
    const MAX_ITERATIONS: usize = 100;

    let n_samples = features.nrows();
    if n_samples == 0 {
        return Err(AllocationError::ClusteringError(
            "cannot cluster an empty feature matrix".to_string(),
        ));
    }

    // With no more samples than clusters, the assignment is trivial and
    // seeding distinct centroids is impossible.
    if n_samples <= N_CLUSTERS {
        return Ok((0..n_samples).collect());
    }

    let mut rng = rand::rng();

    // k-means++ seeding: the first centroid is uniform, and each
    // subsequent one is drawn with probability proportional to its
    // squared distance from the nearest centroid chosen so far. This is
    // what keeps the two centroids apart on the first iteration; picking
    // both uniformly collapses often on clustered data.
    let mut centroids: Vec<Array1<f64>> =
        vec![features.row(rng.random_range(0..n_samples)).to_owned()];

    while centroids.len() < N_CLUSTERS {
        let distances: Vec<f64> = features
            .rows()
            .into_iter()
            .map(|row| {
                centroids
                    .iter()
                    .map(|c| squared_distance(&row.to_owned(), c))
                    .fold(f64::INFINITY, f64::min)
            })
            .collect();

        let total: f64 = distances.iter().sum();
        let next = if total > 0.0 && total.is_finite() {
            // Weighted draw over the squared distances.
            let mut target = rng.random_range(0.0..total);
            distances
                .iter()
                .position(|d| {
                    target -= d;
                    target <= 0.0
                })
                .unwrap_or(n_samples - 1)
        } else {
            // Every point coincides with a centroid (e.g. all-identical
            // rows), so the weights carry no information — fall back to a
            // uniform pick rather than dividing by zero.
            rng.random_range(0..n_samples)
        };
        centroids.push(features.row(next).to_owned());
    }

    let mut assignments = vec![0usize; n_samples];

    for _ in 0..MAX_ITERATIONS {
        // Assignment step.
        let mut changed = false;
        for (i, row) in features.rows().into_iter().enumerate() {
            let point = row.to_owned();
            let nearest = centroids
                .iter()
                .enumerate()
                .map(|(k, c)| (k, squared_distance(&point, c)))
                // `partial_cmp`, not `total_cmp`: the latter needs Rust
                // 1.62 and the workspace MSRV is 1.56. Distances here are
                // non-negative and finite (the inputs are validated by
                // `check_invalid_data!` upstream), so the orderings agree.
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
                .map_or(0, |(k, _)| k);
            if assignments[i] != nearest {
                assignments[i] = nearest;
                changed = true;
            }
        }

        // Update step: each centroid becomes the mean of its members.
        // An empty cluster keeps its previous position rather than
        // becoming NaN.
        for (k, centroid) in centroids.iter_mut().enumerate() {
            let members: Vec<_> = features
                .rows()
                .into_iter()
                .zip(&assignments)
                .filter(|(_, &a)| a == k)
                .map(|(row, _)| row)
                .collect();

            if members.is_empty() {
                continue;
            }

            let mut mean = Array1::<f64>::zeros(features.ncols());
            for row in &members {
                mean += &row.to_owned();
            }
            #[allow(clippy::cast_precision_loss)]
            let count = members.len() as f64;
            mean /= count;
            *centroid = mean;
        }

        if !changed {
            break;
        }
    }

    Ok(assignments)
}

/// Squared Euclidean distance between two equal-length vectors.
///
/// Squared rather than actual distance: the square root is monotonic, so
/// it does not change which centroid is nearest, and skipping it avoids
/// a `sqrt` per comparison.
fn squared_distance(a: &Array1<f64>, b: &Array1<f64>) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum()
}

/// Helper function for sentiment analysis (placeholder).
///
/// This function generates random sentiment scores for demonstration purposes.
/// Replace this function with the actual sentiment analysis logic.
///
/// # Arguments
///
/// * `num_days` - The number of days for which to generate sentiment scores.
///
/// # Returns
///
/// A vector of random sentiment scores (`Vec<f64>`) for the specified number of days, or an error if sentiment analysis fails.
///
/// # Errors
///
/// Returns an error if the sentiment analysis fails.
///
/// # Examples
///
/// ```
/// use nalufx::utils::calculations::get_sentiment_scores;
/// let num_days = 3;
/// let sentiment_scores = get_sentiment_scores(num_days).unwrap();
/// assert_eq!(sentiment_scores.len(), num_days);
/// ```
pub fn get_sentiment_scores(num_days: usize) -> Result<Vec<f64>, String> {
    // Implement the actual sentiment analysis logic here
    // For demonstration purposes, we'll return random scores
    let mut rng = rand::rng();
    let sentiment_scores: Vec<f64> = (0..num_days).map(|_| rng.random_range(0.0..1.0)).collect();
    Ok(sentiment_scores)
}

/// Helper function for reinforcement learning (placeholder).
///
/// This function generates random optimal actions for demonstration purposes.
/// Replace this function with the actual reinforcement learning logic.
///
/// # Arguments
///
/// * `num_days` - The number of days for which to generate optimal actions.
///
/// # Returns
///
/// A vector of random optimal actions (`Vec<f64>`) for the specified number of days, or an error if reinforcement learning fails.
///
/// # Errors
///
/// Returns an error if the reinforcement learning process fails.
///
/// # Examples
///
/// ```
/// use nalufx::utils::calculations::get_optimal_actions;
/// let num_days = 3;
/// let optimal_actions = get_optimal_actions(num_days).unwrap();
/// assert_eq!(optimal_actions.len(), num_days);
/// ```
pub fn get_optimal_actions(num_days: usize) -> Result<Vec<f64>, String> {
    // Implement the actual reinforcement learning logic here
    // For demonstration purposes, we'll return random actions
    let mut rng = rand::rng();
    let optimal_actions: Vec<f64> = (0..num_days).map(|_| rng.random_range(0.0..1.0)).collect();
    Ok(optimal_actions)
}

#[cfg(test)]
mod clustering_tests {
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

    /// All-identical rows have no meaningful split; the k-means++ weights
    /// are all zero, which must not divide by zero or emit NaN labels.
    #[test]
    fn handles_identical_rows() {
        let features = Array2::from_shape_vec((3, 4), vec![0.0; 12]).unwrap();
        let clusters = perform_clustering(&features).unwrap();
        assert_eq!(clusters.len(), 3);
        assert!(clusters.iter().all(|&c| c < 2));
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
        assert!(clusters.iter().all(|&c| c < 2), "out-of-range label: {clusters:?}");
    }
}
