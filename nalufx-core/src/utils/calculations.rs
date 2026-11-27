//! Forecasting and allocation, from [`nalufx_engine`].
//!
//! The maths moved to the `nalufx-engine` crate in 0.0.6. These paths are
//! kept until v0.1.0 so code written against 0.0.5 does not break twice.

pub use nalufx_engine::clustering::perform_clustering;
pub use nalufx_engine::features::extract_features;
pub use nalufx_engine::forecast::{forecast_seasonal, forecast_time_series, FORECAST_LEVEL};
pub use nalufx_engine::horizon::{
    allocate_over_horizon as calculate_optimal_allocation, inverse_volatility, DEFAULT_HORIZON_DAYS,
};
