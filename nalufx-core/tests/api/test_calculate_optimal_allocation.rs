//! Tests for `calculate_optimal_allocation`: input validation, a golden
//! fixture, and the properties every allocation must have.

#[cfg(test)]
mod tests {
    use nalufx::errors::AllocationError;
    use nalufx::utils::calculations::{calculate_optimal_allocation, forecast_time_series};
    use proptest::prelude::*;

    const FIXTURE: &str = include_str!("../fixtures/returns_250.csv");
    const GOLDEN: &str = include_str!("../fixtures/returns_250.weights.json");

    /// The 250 synthetic daily returns of `tests/fixtures/returns_250.csv`.
    fn fixture() -> Vec<f64> {
        FIXTURE.lines().skip(1).map(|l| l.parse().expect("a number per line")).collect()
    }

    /// An allocation or its error, as the JSON the API would send.
    fn as_json(result: &Result<Vec<f64>, AllocationError>) -> String {
        match result {
            Ok(weights) => serde_json::to_string(weights).expect("weights serialise"),
            Err(err) => format!("error: {err}"),
        }
    }

    #[test]
    fn rejects_empty_returns() {
        assert_eq!(calculate_optimal_allocation(&[], 5), Err(AllocationError::EmptyInput));
    }

    #[test]
    fn rejects_nan_and_infinite_returns() {
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let returns = [0.01, bad, 0.02];
            assert_eq!(
                calculate_optimal_allocation(&returns, 5),
                Err(AllocationError::InvalidData)
            );
        }
    }

    #[test]
    fn rejects_a_return_over_100_percent() {
        let returns = [0.02, -0.01, 1.5];
        assert_eq!(calculate_optimal_allocation(&returns, 5), Err(AllocationError::OutlierData));
    }

    #[test]
    fn rejects_a_zero_day_horizon() {
        assert_eq!(
            calculate_optimal_allocation(&fixture(), 0),
            Err(AllocationError::InvalidHorizon)
        );
    }

    #[test]
    fn a_constant_series_has_no_inverse_volatility_weight() {
        let result = calculate_optimal_allocation(&[0.01; 40], 5);
        assert!(
            matches!(
                result,
                Err(AllocationError::ZeroVolatility | AllocationError::ForecastingError(_))
            ),
            "{result:?}"
        );
    }

    /// An all-zero series fits with an interval about 1e-98 wide: noise
    /// around zero, which a floor relative to the interval cannot see.
    /// Found by proptest (it shrank to 30 zeros, 1 day).
    #[test]
    fn an_all_zero_series_has_no_inverse_volatility_weight() {
        for n in [30, 120] {
            assert_eq!(
                calculate_optimal_allocation(&vec![0.0; n], 2),
                Err(AllocationError::ZeroVolatility)
            );
        }
    }

    /// The fixture gives exactly the checked-in weights. They were produced
    /// by this implementation (AutoETS from augurs 0.10.2), so the test
    /// guards against any change in the numbers, not their derivation.
    #[test]
    fn the_250_day_fixture_gives_the_golden_weights() {
        let weights = calculate_optimal_allocation(&fixture(), 6).expect("the fixture allocates");
        let golden: Vec<f64> = serde_json::from_str(GOLDEN).expect("golden weights");
        assert_eq!(weights.len(), golden.len());
        for (day, (w, g)) in weights.iter().zip(&golden).enumerate() {
            assert_eq!(w.to_bits(), g.to_bits(), "day {}: {w} != {g}", day + 1);
        }
    }

    /// The near days get more than the far ones: the interval widens with
    /// the horizon.
    #[test]
    fn nearer_days_get_at_least_as_much() {
        let weights = calculate_optimal_allocation(&fixture(), 6).expect("the fixture allocates");
        assert!(weights.windows(2).all(|w| w[0] >= w[1]), "{weights:?}");
    }

    /// Proptest cases per property: 10,000, or `PROPTEST_CASES` if set.
    fn cases() -> u32 {
        std::env::var("PROPTEST_CASES").ok().and_then(|v| v.parse().ok()).unwrap_or(10_000)
    }

    fn returns() -> impl Strategy<Value = Vec<f64>> {
        prop::collection::vec(-0.05f64..0.05, 30..=120)
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(cases()))]

        /// Two calls on the same input give byte-identical JSON.
        #[test]
        fn allocation_is_deterministic(returns in returns(), days in 1usize..=12) {
            let first = as_json(&calculate_optimal_allocation(&returns, days));
            let second = as_json(&calculate_optimal_allocation(&returns, days));
            prop_assert_eq!(&first, &second);
            let forecast = |r: &[f64]| serde_json::to_string(&forecast_time_series(r, days).ok());
            prop_assert_eq!(forecast(&returns).ok(), forecast(&returns).ok());
        }

        /// Every weight is at least 0, there is one per day, and they sum
        /// to 1 within 1e-9.
        #[test]
        fn weights_are_long_only_and_sum_to_one(returns in returns(), days in 1usize..=12) {
            if let Ok(weights) = calculate_optimal_allocation(&returns, days) {
                prop_assert_eq!(weights.len(), days);
                prop_assert!(weights.iter().all(|w| *w >= 0.0 && w.is_finite()), "{:?}", weights);
                let total: f64 = weights.iter().sum();
                prop_assert!((total - 1.0).abs() <= 1e-9, "sum {}", total);
            }
        }
    }
}
