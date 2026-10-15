# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.0.3] - Unreleased

### Security

- **quinn-proto 0.11.18** for RUSTSEC-2026-0338, -0340 and -0343
  (remote-triggerable panics). It sits in `Cargo.lock` through
  reqwest's optional HTTP/3 stack and no nalufx feature builds it, but
  `cargo audit` failed on the lockfile.

### Dependencies

- lazy_static 1.5.1, hyper 1.12.0, tokio-test 0.4.6, tokio 1.53.2 and
  thiserror 2.0.21, from Dependabot (#51 to #55).

### Changed

- The MSRV stays at Rust 1.92: slint 1.17 needs it. Two code comments
  that still cited a 1.56 floor now say where their choices came from.
- `deny.toml` uses the current cargo-deny schema. cargo-deny 0.20
  refused the file (`unlicensed`, `copyleft`, `allow-osi-fsf-free` and
  the licence `deny` list are gone), so `cargo deny check` could not
  run. The allow list now names the licences the old rules accepted in
  the dependency graph; the policy is unchanged.
- CI runs the shared pipelines security workflow (cargo audit, cargo
  deny, CodeQL and dependency review), as the other crates in the
  family do. cargo deny ran nowhere in CI before.

## [0.0.2] - 2026-10-11

### Changed

- Bumped workspace crates to version 0.0.2.
- Updated dependencies: rand (0.10.3), reqwest (0.13.5), printpdf (0.12.8), yahoo_finance_api (5.0.0), hyper (1.11.1).
- Resolved security advisory RUSTSEC-2026-0285 by upgrading rustls to 0.23.45.

## [0.0.1] - 2024-05-01

### Added

- Initial release of NaluFX foreign exchange and financial analytics library.
