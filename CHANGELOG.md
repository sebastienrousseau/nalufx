# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.12] - 2026-11-27

Observability and operations (M6): every request can be explained
after the fact, and every allocation can be reproduced.

### Added

- **Traces.** Every request runs in a `request` span (method, route,
  request id, key id, status, latency) with child spans for `forecast`,
  `allocate` (method, assets, rows, solver iterations), `report`
  (provider, model, prompt version, tokens) and each provider attempt.
  A key appears only as its id, the first 8 hex characters of its
  SHA-256.
- **Logs through RustLogs (rlg 0.0.19).** JSON lines by default
  (`NALUFX_LOG_FORMAT=text` for logfmt), each carrying the fields of
  its request plus `trace_id` and `span_id`. Every line and exported
  span is scrubbed of card numbers, tokens, emails, IP addresses and
  provider keys.
- **OpenTelemetry.** With `OTEL_EXPORTER_OTLP_ENDPOINT` set, spans go
  to the collector as OTLP traces, linked to the logs.
- **Metrics** at `GET /metrics` (behind an API key): requests and
  latency by route and status, allocation time and solver failures by
  method, and language model requests, tokens and latency by provider
  and model. CI checks the output with Prometheus's `promtool`.
- **Audit log.** With `NALUFX_AUDIT_LOG` set, one JSON line per
  allocation (canonical input and its SHA-256, engine version, method,
  parameters, weights) and per report (input hash, prompt version,
  provider, model, tokens; never the investor's words). A line replays
  through the engine to the same weights, bit for bit. A line that
  cannot be written fails the request with 503.
- **Runbook** (`docs/runbook.md`): running, configuring, key rotation,
  logs, traces, metrics, the audit log, and three alerts with what to
  do about each, shipped as Prometheus rules in `docs/alerts.yml`.
- Releases attach their signed provenance as
  `nalufx-v<version>.intoto.jsonl`, verifiable offline with
  `gh attestation verify <file> --bundle <it> --repo
  sebastienrousseau/nalufx`.

### Security

- A retry warning logged the provider's error reply, which can echo
  the prompt. It now logs the provider and attempt only.
- A failed `/v1/report` answered with the provider's reply, its
  address or the server's settings. Problems now name the provider and
  the kind of failure only.
- `/readyz`, which needs no key, named the provider's address when the
  provider did not answer. It no longer does.
- `.github/SECURITY.md` held a generic policy with no way to report a
  vulnerability, and GitHub showed it instead of the real one. The
  real policy replaces it.

### Changed

- **Breaking (library):** nalufx-llms and nalufx-core log through
  `tracing` instead of `log`.
- **Breaking:** the server logs through rlg, so log lines have rlg's
  shape (`Level`, `Timestamp`, `Description`, `Attributes`), and go to
  journald when the host has it (`RLG_FALLBACK_STDOUT=1` for standard
  output).

## [0.0.11] - 2026-11-27

The release that ships 0.0.10's code. The code is the same as 0.0.10;
only the release workflow changed.

### Fixed

- The Windows release builds failed, so 0.0.10 was tagged but never
  released. A 0.0.10 change made the build steps read `"$TARGET"`,
  which PowerShell, the default shell on Windows runners, sees as an
  empty variable. The build job now runs every step in bash.
- The release workflow now builds every target on any pull request that
  changes it, so a broken release cannot wait until a tag to show up.

## [0.0.10] - 2026-11-27

Security and supply chain (M5): every control comes with a test or a CI
job, so it cannot quietly regress. Tagged but not released: the
Windows builds failed (fixed in 0.0.11, which ships this code).

### Security

- **API keys never print.** The adapters derived `Debug` over the raw
  key, so formatting one with `{:?}` printed it. Keys are now an
  `ApiKey` (wrapping `secrecy::SecretString`): `{:?}` shows
  `[REDACTED]`, there is no `Display`, and the key is wiped from memory
  when dropped.
- **Prompt injection.** Prompt `allocation_report` v2 puts the
  portfolio name, the figures and the investor context inside
  `<portfolio>`, `<facts>` and `<investor_context>` tags and tells the
  model never to follow instructions there. Control, bidirectional and
  zero-width characters are removed, `<` and `>` cannot close a tag,
  and the words are capped at 200 and 2,000 characters. An injection
  corpus test in the OWASP LLM01 style guards it.
- **Denial of service.** Every allocation computed the covariance's
  condition number by Jacobi sweeps that never met their stopping rule
  on wide inputs: 500 assets took 66 seconds whatever the method. The
  eigenvalues now come from nalgebra: 0.1 to 0.5 seconds for 500
  assets.
- **Input limits.** The API refuses, with a 400 naming the limit, more
  than 10,000 history points or rows of returns, more than 500 assets
  and more than 4 seasonal periods. The limits are in the contract.
- **NaN weights.** Subnormal variances (returns near 1e-160) made HRP
  answer NaN weights, which JSON sends as `null`. They are now
  `ZeroVolatility`, and any method's non-finite weights are an error:
  an `Ok` always holds usable weights. Found by fuzzing.
- `unsafe` code is forbidden in every crate.

### Added

- Fuzzing: four cargo-fuzz targets in `fuzz/` (the CSV reader, the API
  request bodies through the real handlers, the allocator and the
  prompt cleaning), one minute each on every pull request and 30
  minutes each every night.
- Releases carry a CycloneDX SBOM for each binary and SLSA build
  provenance for every archive and SBOM, verified in the release job.
  Check one with `gh attestation verify <file> --repo
  sebastienrousseau/nalufx`.
- `cargo vet` in CI: 95 crates audited by Mozilla, Google, the
  Bytecode Alliance, ISRG, Zcash and Embark, the rest exempted as a
  baseline; a new dependency fails until it is reviewed.
- gitleaks scans every pushed commit for secrets.
- OpenSSF Scorecard runs weekly and on every push to `main`.
- `SECURITY.md`: private reporting through GitHub, response times and
  a 90-day disclosure limit.

### Changed

- **Breaking:** reports record prompt `allocation_report` version 2. A
  portfolio name over 200 or a context over 2,000 characters is
  refused (the new `LlmError::Input`: 400 from the API, exit code 3
  from the CLI).
- Every GitHub Action is pinned to a commit. Workflows default to a
  read-only token, and checkouts no longer keep it in `.git/config`.
- `LICENSE-APACHE` carried another project's copyright notice; both
  licences now name Sebastien Rousseau.

## [0.0.9] - 2026-11-27

The `nalufx` command line, brought forward from M7: the engine is now
usable from a terminal without writing code or running a server.

### Added

- **`nalufx`** (the `nalufx-cli` crate), with `--help` on every command:
  - `allocate`: weights by any of the five methods, within bounds.
  - `forecast`: the next days with a 95% interval, seasonal with
    `--seasonal`.
  - `backtest`: one method, or `--method all` (the default) side by side.
  - `report`: a language model's narrative about an allocation, piped
    from `allocate --json` or read from a file.
  - `methods` and `providers`: the choices, and which provider keys are
    set.
- Input from live prices (`--assets SPY,TLT,GLD`) or CSV files (`--file`),
  with synthetic sample data in `nalufx-cli/data` for trying it offline.
- A table by default and `--json` for scripts.
- Errors say what to do. Bounds no weights can meet are rejected before
  any price is fetched, and a missing API key before the input is read.
- Exit codes: `0` success, `2` usage, `3` unusable input, `4` no answer,
  `5` a network, price feed or language model failure, `1` anything
  else.
- End-to-end tests run the binary on the sample files, every error path
  included, and `report` against a local mock provider. A CI coverage
  gate holds nalufx-cli at 80% of lines (measured: 81%).

### Removed

- `POST /predict` (410 Gone in 0.0.8) is gone from the API and its
  contract.
- The interactive example menu (`cargo run --example nalufx`); the
  examples in `nalufx-core/examples/services` remain as library samples.

## [0.0.8] - 2026-11-27

The HTTP API moves to axum and tower in the `nalufx-api` crate: limits,
timeouts, authentication, rate limiting and shutdown now come from
tested middleware instead of a hand-rolled hyper loop.

### Added

- **`nalufx-api`** (library and `nalufx-api` binary): `POST /v1/forecast`,
  `POST /v1/allocate` and `POST /v1/report`, with `GET /healthz` (the
  process is up) and `GET /readyz` (configuration loaded, and the
  language model's endpoint answers if one is configured).
- **Middleware**, outermost first: request id (`x-request-id`, generated
  if absent), compression, problem details, tracing, a 30 s timeout
  (408), a concurrency limit (64 by default; excess requests wait), a
  1 MiB body limit (413), and CORS, denied unless `NALUFX_CORS_ORIGINS`
  names origins.
- **Authentication**: bearer API keys, configured only as SHA-256 hashes
  (`NALUFX_API_KEY_SHA256`) and compared in constant time against every
  stored hash. The server does not start without one. `/healthz` and
  `/readyz` are the only open routes.
- **Rate limiting** per key (token buckets): 600 requests a minute, and
  30 `/v1/report` calls a minute, each answering 429 with `Retry-After`.
- **Graceful shutdown**: SIGTERM or Ctrl-C stops accepting connections
  and drains in-flight requests for up to 25 s.
- **OpenAPI 3.1**, generated from the code with utoipa and committed as
  `nalufx-api/openapi.json`; a test fails when they differ, and CI runs
  Schemathesis 4.33.4 against it on a release build (with a stub
  language model) and fails on any 5xx.
- `GET /openapi.json` serves the committed contract, and `GET /` points
  at it and at the health routes; both are open, like `/healthz` and
  `/readyz`.
- **Problem details**: every non-2xx body is `application/problem+json`
  with a stable `type` under `https://nalufx.com/problems/` and the
  `request_id`, including the responses middleware and axum's
  extractors produce.
- Tests, in process with `tower::ServiceExt::oneshot`: a missing or
  wrong key is 401; an oversize body 413; a slow request 408 at the
  timeout; a request past the concurrency limit waits; over a key's
  rate is 429; CORS, request ids, compression and every route. A
  separate test sends the process SIGTERM with a request in flight,
  which completes with 200.

### Breaking

- `POST /predict` answers 410 Gone, pointing at `/v1/allocate`, and is
  removed in 0.0.9.
- nalufx-core no longer has an HTTP server: its `api` and `config`
  modules, the `models::cash_flow_dm` request and response types and
  the `nalufx-core` binary are removed. Install `nalufx-api` instead.
- `LlmProvider` has a new required method, `base_url`.

### Fixed

- Found by running the server: with a client sending
  `Accept-Encoding: gzip`, a problem's detail was replaced by
  "malformed problem", because compression ran inside the problem
  layer. A regression test covers it.

## [0.0.7] - 2026-11-27

nalufx-llms is rebuilt around one provider trait: seven copy-pasted
OpenAI-shaped modules give way to native Anthropic and Gemini adapters
and one OpenAI-compatible adapter, with timeouts, retries and
schema-checked reports. Language models write words only.

### Added

- `LlmProvider::complete(&ChatRequest) -> Result<ChatResponse, LlmError>`,
  with `ChatResponse { text, usage, model, provider }`.
- Adapters: `Anthropic` (Messages API: `x-api-key`, `anthropic-version`,
  `content[].text`), `Gemini` (`generateContent`: `x-goog-api-key`,
  `candidates[].content.parts[].text`), and `OpenAiCompatible` with
  presets for OpenAI, Mistral, Ollama and Agy, or any base URL (vLLM,
  llama.cpp).
- `LlmConfig`: the model, token limit and temperature are the caller's.
  There is no default model in the library.
- One shared client (`http::Http`) with 5 s connect and 60 s request
  timeouts; a timeout, transport failure, `429` or `5xx` is retried up
  to 3 attempts with exponential backoff and jitter, honouring
  `Retry-After`. A `4xx` or a malformed body is not retried.
- `registry::from_env(name, config, http)`, which builds a provider by
  name with its key from the environment or `.env`, and an optional
  `NALUFX_LLM_BASE_URL` override.
- Prompts as versioned templates in `nalufx-llms/prompts/`; every
  report records its prompt's name and version.
- `report::generate`: the model gets the computed figures and a JSON
  Schema generated with schemars from `AllocationReport`, and its reply
  must deserialise into that type exactly (no missing or unknown
  fields) and pass its checks, or it is an `LlmError::Schema`.
- Tests: a contract suite per adapter against recorded replies served by
  wiremock (request headers and body, success, a 429 with Retry-After,
  a 500 retried three times, a 400 not retried, a timeout, a malformed
  body, a reply with no text); the M3 exit gate, a valid report through
  each adapter kind and a schema error for an off-schema reply; and a
  nightly, opt-in `llm-live` workflow that calls the real providers
  when their keys and models are set as repository secrets. No other
  test reaches a provider.
- CI coverage gate for nalufx-llms at 90% of lines (measured: 98%).

### Breaking

- The `LLM` trait (returning `serde_json::Value` and `reqwest::Error`),
  the `send_*_request`, `parse_*_response` and `get_*_api_key`
  functions and the `models` module are removed; use `LlmProvider`,
  an adapter or `registry::from_env`.
- `automated_cash_allocation_svc::generate_analysis` and
  `bellwether_stock_analysis_svc::generate_analysis` take
  `&dyn LlmProvider` and return `NaluFxError` (now with `Llm` and
  `Allocation` variants) instead of `Box<dyn Error>`; the first returns
  a validated `Report`.
- `LlmError` has new variants (`Transport`, `Status`, `Schema`,
  `Config`) and `Request` is gone.

### Fixed

- The "claude" and "gemini" adapters sent OpenAI-shaped requests with a
  bearer token and read `choices[0]`, which neither API returns.
- Reports read `["choices"][0]["message"]["content"]` and became an
  empty string when it was missing.
- The prompt and the model (`gpt-3.5-turbo`, `max_tokens: 100`) were
  hard-coded.

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
  at p95, for every method (measured on the CI runner: at most 0.5 ms),
  and criterion benchmarks.
- A CI coverage gate (cargo-llvm-cov): at least 90% of nalufx-engine's
  lines and 80% of the workspace's (measured: 96% and 82%).
- Ledoit-Wolf is cross-checked against scikit-learn 1.7.2 on a 50 x 5
  matrix (shrinkage, diagonal and first row within 1e-12); the
  reference script is `nalufx-engine/tests/fixtures/ledoit_wolf_reference.py`.

### Breaking

- `/predict` allocates across assets instead of across the days of one
  series: `daily_returns` is replaced by `assets` and `returns`, and the
  response's `optimal_allocation` by `allocation`.
- The 0.0.5 single-series allocator is `nalufx_engine::horizon::allocate_over_horizon`;
  `nalufx::utils::calculations::calculate_optimal_allocation` still
  names it, and the other moved items keep their `nalufx` paths, until
  v0.1.0.

### Fixed

- `validate_positive_float` had its two error messages swapped: a
  negative number said "not a valid float", and text said "not
  positive".
- `format_currency` printed `-$0.00` for a negative value that rounds to
  zero; the sign now follows the rounded cents.
- `tests/utils/test_input.rs` tested copies of the input functions;
  they now call `read_line_from`, which `get_input` reads through.

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
  noisy, and plain mean-variance often trails equal weight out of
  sample (DeMiguel, Garlappi and Uppal, 2009). Minimum variance (2.63),
  risk parity (2.19) and HRP (2.07) beat it. The README and the
  `MeanVariance` docs say so.

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
