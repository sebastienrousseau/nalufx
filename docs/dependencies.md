# Dependency decisions

AGENTS.md §11: a new third-party dependency needs the maintainer's
explicit approval of that named crate, recorded here with the reason.

| Date | Crate | Lockfile gains | Why not in-house | Decision |
| --- | --- | --- | --- | --- |
| 2026-11-28 | `toml` 1.2.3 (`std`, `parse`, `serde`; no defaults) in nalufx-api | 6: `toml`, `toml_parser`, `toml_datetime`, `toml_writer`, `serde_spanned`, `winnow`, all published by Ed Page (toml-rs); no RustSec advisories | A TOML parser is a parser of untrusted input with a long spec; the toml-rs stack is the ecosystem's reference and is trusted by the Mozilla, ISRG and Bytecode Alliance audit sets | Approved by the maintainer. cargo-vet exemptions, as for the other unaudited crates. |
| 2026-11-28 | `petitcarnets` 0.0.7 (default `bubbles` feature) in nalufx-cli | 1: `petitcarnets`, which has no dependencies; published by the maintainer | A terminal UI runtime (raw mode, input decoding, a diffing renderer, terminal restore on every exit) plus list, table, text input, spinner and help components is a library of its own; petitcarnets is that library, maintained by the same person, and `nalufx tui` is its planned dogfood (its milestone v0.0.8) | Approved by the maintainer. A cargo-vet exemption, as for the other unaudited crates. |
