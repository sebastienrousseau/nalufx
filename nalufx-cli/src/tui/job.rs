//! What `nalufx tui` runs: an allocation or forecasts of the chosen
//! tickers, through the same functions as `nalufx allocate` and
//! `nalufx forecast`.

use crate::cli::{Bounds, Method, ReturnsSource};
use crate::commands::{allocation, check_allocation, forecast_series};
use crate::error::CliError;
use crate::input::{self, Returns};
use nalufx_engine::allocate::Allocation;
use nalufx_engine::forecast::IntervalForecast;
use ndarray::{Array2, Axis};

/// Where the returns come from.
#[derive(Debug, Clone)]
pub enum Source {
    /// A CSV read once at start: every column is a ticker to pick from.
    File {
        /// The column names.
        assets: Vec<String>,
        /// One row per day, one column per asset.
        returns: Array2<f64>,
    },
    /// Daily prices fetched for the chosen tickers when a job runs.
    Live {
        /// The tickers to pick from.
        tickers: Vec<String>,
    },
}

impl Source {
    /// The source `nalufx tui` was given; a file is read and checked now,
    /// before the interface starts.
    ///
    /// # Errors
    ///
    /// [`CliError::Input`] if the file cannot be read or is not returns.
    pub fn from_args(source: &ReturnsSource) -> Result<Self, CliError> {
        match &source.file {
            Some(path) => {
                let (assets, returns) = input::read_returns(path)?;
                Ok(Self::File { assets, returns })
            },
            None => Ok(Self::Live { tickers: source.assets.clone() }),
        }
    }

    /// The tickers to pick from.
    #[must_use]
    pub fn tickers(&self) -> &[String] {
        match self {
            Self::File { assets, .. } => assets,
            Self::Live { tickers } => tickers,
        }
    }

    /// The returns of `chosen`, in the source's order.
    fn returns(&self, chosen: &[String]) -> Result<Returns, CliError> {
        match self {
            Self::File { assets, returns } => Ok(columns(assets, returns, chosen)),
            Self::Live { .. } => fetch(chosen),
        }
    }
}

/// The columns of `returns` named in `chosen`, row-major as a file reads
/// (`select` along columns gives column-major, and the engine's sums in
/// another order differ in the last bit from `nalufx allocate`'s).
fn columns(assets: &[String], returns: &Array2<f64>, chosen: &[String]) -> Returns {
    let picked: Vec<usize> = (0..assets.len()).filter(|&i| chosen.contains(&assets[i])).collect();
    let selected = returns.select(Axis(1), &picked).as_standard_layout().into_owned();
    (picked.iter().map(|&i| assets[i].clone()).collect(), selected)
}

/// Prices for `tickers` as `nalufx allocate --assets` fetches them, on a
/// runtime of the job's own: the job runs on the interface's task
/// thread, outside the command's runtime.
fn fetch(tickers: &[String]) -> Result<Returns, CliError> {
    let runtime =
        tokio::runtime::Builder::new_current_thread().enable_all().build().map_err(|e| {
            CliError::Runtime { message: format!("cannot start the fetch: {e}"), hint: None }
        })?;
    let source = ReturnsSource { assets: tickers.to_vec(), file: None };
    runtime.block_on(input::returns(&source))
}

/// What to run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Job {
    /// `nalufx allocate --method`.
    Allocate(Method),
    /// `nalufx forecast --days` of each chosen ticker's returns.
    Forecast {
        /// Days to forecast.
        days: u16,
    },
}

/// A job and its settings.
#[derive(Debug, Clone)]
pub struct Request {
    /// What to run.
    pub job: Job,
    /// The chosen tickers.
    pub tickers: Vec<String>,
    /// The weight bounds and the covariance estimator.
    pub bounds: Bounds,
    /// Risk aversion, for mean-variance.
    pub risk_aversion: Option<f64>,
}

/// A job's result.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome {
    /// The weights.
    Weights(Allocation),
    /// Each ticker's forecast.
    Forecasts(Vec<(String, IntervalForecast)>),
}

/// Runs `request` on `source`, checking the method and bounds before any
/// prices are fetched, as `nalufx allocate` does.
///
/// # Errors
///
/// The [`CliError`] the command would have exited with.
pub fn run(source: &Source, request: &Request) -> Result<Outcome, CliError> {
    match request.job {
        Job::Allocate(method) => {
            let Request { bounds, risk_aversion, .. } = *request;
            check_allocation(method, risk_aversion, &bounds, request.tickers.len())?;
            let (assets, returns) = source.returns(&request.tickers)?;
            allocation(method, risk_aversion, &bounds, assets, returns).map(Outcome::Weights)
        },
        Job::Forecast { days } => {
            let (assets, returns) = source.returns(&request.tickers)?;
            let forecast = |(asset, series): (String, Vec<f64>)| {
                forecast_series(&series, days, &[]).map(|f| (asset, f))
            };
            let series = returns.columns().into_iter().map(|c| c.to_vec());
            assets
                .into_iter()
                .zip(series)
                .map(forecast)
                .collect::<Result<_, _>>()
                .map(Outcome::Forecasts)
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn columns_are_in_standard_layout() {
        let assets: Vec<String> = ["A", "B"].map(String::from).to_vec();
        let returns = ndarray::array![[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]];
        let (_, picked) = columns(&assets, &returns, &assets);
        assert!(picked.is_standard_layout(), "{:?}", picked.strides());
    }

    #[test]
    fn columns_keep_the_sources_order() {
        let assets: Vec<String> = ["A", "B", "C"].map(String::from).to_vec();
        let returns = ndarray::array![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let (names, picked) = columns(&assets, &returns, &["C".into(), "A".into()]);
        assert_eq!(names, ["A", "C"]);
        assert_eq!(picked, ndarray::array![[1.0, 3.0], [4.0, 6.0]]);
    }
}
