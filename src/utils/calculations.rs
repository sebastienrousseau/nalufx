use augurs_ets::AutoETS;

use crate::errors::AllocationError;
use linfa::prelude::Predict as LinfaPredict;
use linfa::prelude::*;
use linfa_clustering::KMeans;
use ndarray::prelude::*;
use rand::Rng;

/// Calculates the optimal allocation based on daily returns and cash flows.
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
pub fn calculate_optimal_allocation(
    daily_returns: &[f64],
    cash_flows: &[f64],
    market_indices: &[f64],
    fund_characteristics: &[f64],
    num_days: usize,
) -> Result<Vec<f64>, AllocationError> {
    // Check if input slices have the same length
    if daily_returns.len() != cash_flows.len() {
        return Err(AllocationError::InputMismatch);
    }

    // Check if input slices are empty
    if daily_returns.is_empty() || cash_flows.is_empty() {
        return Err(AllocationError::EmptyInput);
    }

    // Check for missing data or invalid values
    if daily_returns.iter().any(|&x| x.is_nan() || x.is_infinite())
        || cash_flows.iter().any(|&x| x.is_nan() || x.is_infinite())
    {
        return Err(AllocationError::InvalidData);
    }

    // Check for outliers (e.g., values outside a certain range)
    let return_threshold = 1.0; // Adjust the threshold as per your requirements
    let cash_flow_threshold = 1_000_000.0; // Adjust the threshold as per your requirements
    if daily_returns.iter().any(|&x| x.abs() > return_threshold)
        || cash_flows.iter().any(|&x| x.abs() > cash_flow_threshold)
    {
        return Err(AllocationError::OutlierData);
    }

    // Feature Engineering
    let features = extract_features(
        daily_returns,
        cash_flows,
        market_indices,
        fund_characteristics,
    )?;

    // Time Series Forecasting
    let forecasted_returns = match forecast_time_series(daily_returns, num_days) {
        Ok(returns) => returns,
        Err(err) => return Err(AllocationError::ForecastingError(err)),
    };
    let forecasted_cash_flows = match forecast_time_series(cash_flows, num_days) {
        Ok(cash_flows) => cash_flows,
        Err(err) => return Err(AllocationError::ForecastingError(err)),
    };

    // Sentiment Analysis
    let sentiment_scores = match analyze_sentiment(num_days) {
        Ok(scores) => scores,
        Err(err) => return Err(AllocationError::SentimentAnalysisError(err)),
    };

    // Reinforcement Learning
    let optimal_actions = match train_reinforcement_learning(num_days) {
        Ok(actions) => actions,
        Err(err) => return Err(AllocationError::ReinforcementLearningError(err)),
    };

    // Clustering
    let clusters = match perform_clustering(&features) {
        Ok(clusters) => clusters,
        Err(err) => return Err(AllocationError::ClusteringError(err.to_string())),
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
            let cluster = clusters[day - 1];

            // Incorporate sentiment score, optimal action, and cluster into the prediction
            let prediction = predicted_return
                * predicted_cash_flow
                * sentiment_score
                * optimal_action
                * (cluster as f64 + 1.0);
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
        return Ok(Vec::new());
    }

    // Normalize predictions to get the optimal allocations
    Ok(predictions
        .into_iter()
        .map(|p| p / total_prediction)
        .collect())
}

fn extract_features(
    daily_returns: &[f64],
    cash_flows: &[f64],
    market_indices: &[f64],
    fund_characteristics: &[f64],
) -> Result<Array2<f64>, AllocationError> {
    let n = daily_returns.len();
    if n != cash_flows.len() || n != market_indices.len() || n != fund_characteristics.len() {
        return Err(AllocationError::InputMismatch);
    }

    let mut features = Array2::<f64>::zeros((n, 4));
    for i in 0..n {
        features[[i, 0]] = daily_returns[i];
        features[[i, 1]] = cash_flows[i];
        features[[i, 2]] = market_indices[i];
        features[[i, 3]] = fund_characteristics[i];
    }

    // Normalize the features
    let mean = features.mean_axis(Axis(0)).unwrap();
    let std_dev = features.std_axis(Axis(0), 0.0);
    features -= &mean;
    features /= &std_dev;

    Ok(features)
}

// Time series forecasting using augurs-ets
fn forecast_time_series(data: &[f64], num_days: usize) -> Result<Vec<f64>, String> {
    let mut search = AutoETS::new(1, "ZZN").map_err(|e| e.to_string())?;
    let model = search.fit(data).map_err(|e| e.to_string())?;
    let forecast = model.predict(num_days, 0.95);
    Ok(forecast.point)
}

// Sentiment analysis using the helper function
pub fn analyze_sentiment(num_days: usize) -> Result<Vec<f64>, String> {
    // Call the sentiment analysis helper function
    let sentiment_scores = match get_sentiment_scores(num_days) {
        Ok(scores) => scores,
        Err(err) => return Err(err.to_string()),
    };
    Ok(sentiment_scores)
}

// Reinforcement learning using the helper function
pub fn train_reinforcement_learning(num_days: usize) -> Result<Vec<f64>, String> {
    // Call the reinforcement learning helper function
    let optimal_actions = match get_optimal_actions(num_days) {
        Ok(actions) => actions,
        Err(err) => return Err(err.to_string()),
    };
    Ok(optimal_actions)
}

// Clustering using K-means with hyperparameter tuning
pub fn perform_clustering(features: &Array2<f64>) -> Result<Vec<usize>, AllocationError> {
    // Convert features to a Dataset
    let dataset = Dataset::from(features.clone());
    // Create the KMeans model with 2 clusters
    let model = match KMeans::params(2).fit(&dataset) {
        Ok(model) => model,
        Err(err) => return Err(AllocationError::ClusteringError(err.to_string())),
    };
    // Predict the clusters for each feature vector
    let clusters = model.predict(&dataset);
    // Convert the clusters to a Vec<usize> and return
    Ok(clusters.to_vec())
}

// Helper function for sentiment analysis (placeholder)
fn get_sentiment_scores(num_days: usize) -> Result<Vec<f64>, String> {
    // Implement the actual sentiment analysis logic here
    // For demonstration purposes, we'll return random scores
    let mut rng = rand::thread_rng();
    let sentiment_scores: Vec<f64> = (0..num_days).map(|_| rng.gen_range(0.0..1.0)).collect();
    Ok(sentiment_scores)
}

// Helper function for reinforcement learning (placeholder)
fn get_optimal_actions(num_days: usize) -> Result<Vec<f64>, String> {
    // Implement the actual reinforcement learning logic here
    // For demonstration purposes, we'll return random actions
    let mut rng = rand::thread_rng();
    let optimal_actions: Vec<f64> = (0..num_days).map(|_| rng.gen_range(0.0..1.0)).collect();
    Ok(optimal_actions)
}
