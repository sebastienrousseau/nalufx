# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
