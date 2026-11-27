# Security policy

## Supported versions

nalufx is before 1.0. Only the latest release receives security fixes;
upgrade to it before reporting.

## Reporting a vulnerability

Please report privately, not in a public issue:

1. Go to <https://github.com/sebastienrousseau/nalufx/security/advisories/new>.
2. Describe the problem, the version or commit, and the steps to
   reproduce it. A failing test or a minimal request is ideal.

What to expect:

- an acknowledgement within 3 working days;
- an assessment, with a severity, within 10 working days;
- a fix in a release, credited to you unless you prefer otherwise;
- coordinated disclosure: the advisory is published when the fix is
  released, and at the latest 90 days after your report.

## Scope

In scope: the `nalufx` command line, the HTTP API (`nalufx-api`), the
engine (`nalufx-engine`) and the language model adapters
(`nalufx-llms`), including:

- leaks of API keys or other secrets;
- prompt injection that changes the report's figures or format, or
  escapes the investor-context delimiters;
- inputs that crash a process or keep it busy far beyond the documented
  limits;
- authentication, rate limiting or CORS bypasses in the API;
- tampered or unverifiable release artefacts.

Out of scope: the content of a language model's prose (every figure is
computed by the engine and checked against a schema), findings that need
an attacker who already controls the machine or the API keys, and
reports from automated scanners without a demonstrated impact.

## How releases are protected

- Signed tags, and SLSA build provenance and an SBOM on every release.
- Every GitHub Action pinned to a commit.
- `cargo audit`, `cargo deny` and CodeQL on every change; Dependabot for
  updates.
- `unsafe` code forbidden in every crate.

nalufx is for research and analysis only, and is not investment advice.
