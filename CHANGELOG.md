# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.6] - 2026-11-27

The maths moves into a new pure crate, `nalufx-engine`, and the interim
near-equal weights give way to constrained optimisers across assets,
each tested against a walk-forward backtest baseline.

### Added

- **nalufx-engine**, with no I/O and no async runtime: forecasting,
  covariance, allocation, backtesting, screening and clustering.
- **Allocation methods** behind one `Allocator` trait, picked by name
  (`by_name`), each long-only with a budget of 1, a per-asset cap and an
  optional floor, and each `Allocation` recording its method,
  parameters and diagnostics (solver status, iterations, condition
  number, shrinkage, whether it was projected onto the bounds):
  - `equal_weight`, the baseline.
  - `min_variance` and `mean_variance` (risk aversion as a parameter),
    quadratic programmes solved with clarabel 0.11.
  - `risk_parity`, equal risk contribution by Newton's method on the
    convex log-barrier formulation.
  - `hrp`, hierarchical risk parity (single linkage on correlation
    distance, recursive bisection), with no matrix inversion.
  Risk parity and HRP have no native form for bounds, so their weights
  are projected onto them (the nearest feasible weights) when they
  break one, and the diagnostics say so.
- **Covariance**: sample, or Ledoit-Wolf shrinkage towards a scaled
  identity (the default), with the intensity scikit-learn computes.
- **Backtests** (`backtest::run`): walk-forward on a trailing lookback,
  rebalancing on a schedule, net of a cost in basis points, reporting
  annualised return, volatility, Sharpe, maximum drawdown and turnover.
- **Seasonal forecasts** (`forecast_seasonal`): MSTL with an AutoETS
  trend, for cash flows with weekly, monthly or other cycles.
- **Screening** (`screening::screen`) of assets by a caller-supplied
  score, such as an ESG rating.
- `POST /predict` takes `assets`, a `returns` matrix, `method`,
  `risk_aversion`, `constraints`, `estimator` and `seasonal_periods`,
  and answers with the forecast and an `allocation`.
- Tests: property tests (10,000 cases) that every method's weights are
  within the bounds and sum to 1 within 1e-9, and are bit-identical on
  a second run; known answers (variances 1 and 4 give minimum-variance
  weights 0.8 and 0.2; risk parity's contributions are equal within
  1e-6); infeasible bounds give a typed error, never weights; a golden
  backtest report per method on a 5-asset, 500-day synthetic fixture.
- A CI performance budget: a 20-asset, 250-day allocation under 50 ms
  at p95, for every method (measured: under 0.4 ms), and criterion
  benchmarks.

### Breaking

- `/predict` allocates across assets instead of across the days of one
  series: `daily_returns` is replaced by `assets` and `returns`, and the
  response's `optimal_allocation` by `allocation`.
- The 0.0.5 single-series allocator is `nalufx_engine::horizon::allocate_over_horizon`;
  `nalufx::utils::calculations::calculate_optimal_allocation` still
  names it, and the other moved items keep their `nalufx` paths, until
  v0.1.0.

### Changed

- The mean-variance, risk parity and ESG examples call the library: the
  mean-variance example used to return equal weights from a
  placeholder, and the ESG example gave every fund a made-up rating of
  4.5. The ESG example now takes your scores.
- `fetch_data_svc::fetch_returns` and `processing_svc::align_returns`
  build a returns matrix from several tickers.
- The README's feature list and examples match the methods that exist;
  its Rust examples run as doc tests.

### Notes

- On the fixture, mean-variance (risk aversion 10) has a lower Sharpe
  ratio than equal weight (1.49 against 1.52): estimated means are
  noisy. Minimum variance (2.63), risk parity (2.19) and HRP (2.07)
  beat it.

### Dependencies

- clarabel 0.11.1 and augurs-mstl 0.10.3 (new); augurs-core and
  augurs-ets 0.10.3; criterion 0.8.2 for benchmarks. ndarray-stats,
  which nothing uses now, is removed.

## [0.0.5] - 2026-11-27

Every number NaluFX returns is now computed or an explicit error: the
random sentiment and reinforcement-learning stubs are gone, and the
same input always gives the same output.

### Breaking

- `calculate_optimal_allocation(daily_returns, num_days)` takes the
  returns and the horizon only. It returns inverse-volatility weights
  over the horizon: each day weighted by the inverse of its 95% AutoETS
  forecast interval's width, long-only, summing to 1. The K-means
  multiplier and the random factors are no longer in the formula. This
  is the interim method until the M2 optimiser: for daily returns that
  look like noise, AutoETS picks a model whose interval barely widens,
  so the weights come out close to equal.
- `forecast_time_series` returns an `IntervalForecast` (point, lower,
  upper, level) and an `AllocationError` instead of a bare point
  forecast and a `String`.
- `POST /predict` takes `historical_data` and `daily_returns` only, and
  answers `{"forecast": {...}, "optimal_allocation": [...]}`. It
  forecasts with AutoETS instead of asking OpenAI, so it needs no API
  key and makes no network call.
- Error responses are RFC 9457 problem details
  (`application/problem+json`, `ProblemDetails`), replacing
  `{"error": "..."}` (`ErrorResponse`). Input that cannot be used is
  `400`; valid input that cannot be forecast or weighted is `422`.
- `calculate_daily_returns` returns a `Result`: a zero, negative or
  non-finite close is `AllocationError::InvalidPrice` instead of an
  infinite return.
- `perform_clustering` moved to `utils::clustering` (still re-exported
  from `utils::calculations`).

### Removed

- `analyze_sentiment`, `get_sentiment_scores`,
  `train_reinforcement_learning` and `get_optimal_actions`, which
  returned random numbers, with `SentimentAnalysisError` and
  `ReinforcementLearningError`, and the report sections built on them.
- `/predict`'s substitution of six zeros for a forecast of the wrong
  length.
- `rand` from nalufx-core's dependencies.
- The README's sentiment analysis and reinforcement learning features,
  its crates.io install snippet and badges (no `nalufx` crate is
  published), and its usage examples for functions that do not exist.

### Fixed

- K-means clustering is deterministic: farthest-point seeding with
  ties to the lowest index, instead of a random k-means++ draw.
- The diversified ETF report selected the "best" ETF by average
  allocation, which is 1/n for every ETF once normalised, so the pick
  was arbitrary. It now selects the highest mean-to-deviation ratio of
  daily returns and says so. It also no longer truncates every price
  history to 8 days.
- A start price of 0 in the cash allocation report is an error instead
  of an infinite return.
- `normalize_features!` centres a constant column instead of dividing
  it into NaN, and leaves an empty matrix alone instead of panicking.
- A constant return series (all zeros included) is
  `ZeroVolatility`: its forecast interval is rounding noise, which the
  allocation used to weight by.
- The router tests call the production `read_json`, now generic over
  the body type, instead of a copy of it.

### Added

- Property tests (proptest, 10,000 cases each): the same input gives
  byte-identical allocation and forecast JSON; weights are at least 0
  and sum to 1 within 1e-9.
- A golden test: a 250-day synthetic fixture
  (`nalufx-core/tests/fixtures/`, regenerated by its script) gives the
  checked-in weights bit for bit.
- A CI complexity gate (`scripts/complexity-gate.py`): cyclomatic 10,
  cognitive 15, Halstead difficulty 30, 60 lines per function, 500 per
  file, against a committed baseline that only shrinks. Five of the six
  functions over a ceiling, and the one file, are now under; the
  remaining one is `utils::date::parse_ymd`.

### Dependencies

- hyper 1.13.0, from Dependabot (#75).
- `deny.toml` no longer allows 0BSD, NCSA and Unicode-DFS-2016, which
  no dependency uses.

## [0.0.4] - 2026-11-26

### Fixed

- `cargo deny check` passes again: `deny.toml` allows
  CDLA-Permissive-2.0, the licence of `webpki-root-certs` (Mozilla's
  root CA data), which reqwest's rustls platform verifier brings in.
- CI runs `cargo deny check` as a blocking job. The shared security
  workflow ran it with continue-on-error, so the rejected licence above
  never failed a build.

### Dependencies

- env_logger 0.11.12, from Dependabot (#73).

## [0.0.3] - 2026-11-22

### Security

- **quinn-proto 0.11.18** for RUSTSEC-2026-0338, -0340 and -0343
  (remote-triggerable panics). It sits in `Cargo.lock` through
  reqwest's optional HTTP/3 stack and no nalufx feature builds it, but
  `cargo audit` failed on the lockfile.
- **figlet-rs 0.2.0** in place of 1.0.0, which depends on zip 0.6.6
  (advised; zip 0.6 has no fixed release). 0.2.0 has no dependencies.
- pulldown-cmark 0.13.6 (RUSTSEC-2026-0414, -0433), rustls-webpki
  0.103.16 (RUSTSEC-2026-0382) and webpki-root-certs 1.0.10
  (RUSTSEC-2026-0383).
- **chrono removed** (RUSTSEC-2026-0543, unmaintained, no fixed
  release). nalufx-core now uses the `time` crate, already in the
  dependency graph through reqwest and yahoo_finance_api.
- **Slint GUI removed**, and with it pin-utils (RUSTSEC-2026-0441,
  unmaintained, reached through Slint's spin_on) and the rest of the
  Slint tree.
- `.cargo/audit.toml` is gone. Every advisory it ignored (bincode,
  paste, rustybuzz, ttf-parser, lru and the three zbus-family crates)
  came in through Slint or printpdf, and `cargo audit --deny warnings`
  passes with no ignore list.

### Removed

- **Breaking:** the `nalufx-ui` crate and its `nalufx-ui` binary, the
  Slint desktop and WebAssembly front end. It had no features of its
  own beyond the Slint shell, and Slint carried the two unmaintained
  crates above. The CI and release workflows no longer install the
  fontconfig and xkbcommon packages it needed on Linux.

### Dependencies

- lazy_static 1.5.1, hyper 1.12.0, tokio-test 0.4.6, tokio 1.53.2 and
  thiserror 2.0.21, from Dependabot (#51 to #55).
- augurs-ets 0.10.3 (#66), printpdf 0.12.9 (#68) and rand 0.10.4
  (#69), from Dependabot.
- The shared pipelines `security.yml` workflow pin moves to
  `3f9978a` (#67).
- bytes 1.12.2 in `Cargo.lock` (#71, from Dependabot).
- cookie 0.18.3 in `Cargo.lock` (RUSTSEC-2026-0594), reached through
  reqwest's cookie store.
- Workspace crates bumped to version 0.0.3.
- quinn-proto 0.11.19 (RUSTSEC-2026-0523, -0524), quinn-udp 0.5.16
  (RUSTSEC-2026-0538), url 2.5.9 (RUSTSEC-2026-0592) and rustix 1.1.6
  (RUSTSEC-2026-0486) in `Cargo.lock`.
- Removed dependencies that no code used, flagged by Cargo's
  `unused_dependencies` lint: printpdf and tokio-test from nalufx-core
  (ndarray-stats moves to its dev-dependencies, as only examples use
  it), nalufx-core from nalufx-api, nalufx-cli and nalufx-ui, and
  dotenvy, env_logger and futures from nalufx-ui. Dropping printpdf
  takes its azul-layout tree out of `Cargo.lock`.
- futures removed from nalufx-core: no code uses it, and cargo 1.100
  reports it as an unused dependency.

### Changed

- **Breaking:** date values in the nalufx-core API are
  `time::OffsetDateTime` rather than `chrono::DateTime<Utc>`:
  `utils::date::validate_date` returns one, `fetch_data_svc::fetch_data`
  takes them, and `NaluFxError::DateParseError` wraps
  `time::error::Parse`. Accepted inputs and the dates they produce are
  unchanged: a characterisation test fixes the parsing rules, and a
  differential run of two million inputs and one million timestamps
  against chrono found no difference. The new
  `utils::date::format_date` renders `YYYY-MM-DD` as chrono's
  `%Y-%m-%d` did.
- **Breaking:** `utils::ascii::load_standard_font` now returns
  `figlet_rs::FIGfont` rather than `figlet_rs::FIGlet`, and ASCII art is
  rendered at full glyph width: figlet-rs 0.2 does not smush adjacent
  glyphs.
- The MSRV stays at Rust 1.92, which slint 1.17 needed. With Slint
  gone the highest floor a dependency declares is nalgebra's 1.89; the
  MSRV is not lowered because no older toolchain is tested. Two code
  comments that still cited a 1.56 floor now say where their choices
  came from, and the README, which still said 1.56, now says 1.92.
- `deny.toml` uses the current cargo-deny schema. cargo-deny 0.20
  refused the file (`unlicensed`, `copyleft`, `allow-osi-fsf-free` and
  the licence `deny` list are gone), so `cargo deny check` could not
  run. The allow list now names the licences the old rules accepted in
  the dependency graph; the policy is unchanged.
- CI runs the shared pipelines security workflow (cargo audit, cargo
  deny, CodeQL and dependency review), as the other crates in the
  family do. cargo deny ran nowhere in CI before.
- The release workflow quotes `$GITHUB_ENV` (shellcheck SC2086).

## [0.0.2] - 2026-10-11

### Changed

- Bumped workspace crates to version 0.0.2.
- Updated dependencies: rand (0.10.3), reqwest (0.13.5), printpdf (0.12.8), yahoo_finance_api (5.0.0), hyper (1.11.1).
- Resolved security advisory RUSTSEC-2026-0285 by upgrading rustls to 0.23.45.

## [0.0.1] - 2024-05-01

### Added

- Initial release of NaluFX foreign exchange and financial analytics library.
