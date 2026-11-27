//! # nalufx-engine
//!
//! The deterministic maths behind NaluFX: forecasts with prediction
//! intervals, covariance estimation, constrained allocation across
//! assets, and walk-forward backtests.
//!
//! The engine does no I/O and has no async runtime, so every result is
//! reproducible from its inputs alone: the same input always gives the
//! same output, bit for bit, and anything that cannot be computed is an
//! [`AllocationError`] rather than a substitute value.
//!
//! ```
//! use nalufx_engine::allocate::{by_name, AllocationInput, Constraints};
//! use nalufx_engine::covariance::CovarianceEstimator;
//! use ndarray::array;
//!
//! let input = AllocationInput {
//!     assets: vec!["BONDS".into(), "STOCKS".into()],
//!     returns: array![[0.001, 0.012], [0.002, -0.010], [-0.001, 0.015], [0.001, -0.004], [0.000, 0.008]],
//!     constraints: Constraints::default(),
//!     estimator: CovarianceEstimator::LedoitWolf,
//! };
//! let allocation = by_name("min_variance", None)?.allocate(&input)?;
//! assert!(allocation.weights[0] > allocation.weights[1]); // bonds are calmer
//! assert!((allocation.weights.iter().sum::<f64>() - 1.0).abs() < 1e-9);
//! # Ok::<(), nalufx_engine::AllocationError>(())
//! ```

pub mod allocate;
pub mod backtest;
pub mod clustering;
pub mod covariance;
pub mod features;
pub mod forecast;
pub mod horizon;
pub mod linalg;
pub mod screening;
pub mod validate;

mod error;

pub use error::AllocationError;
