# nalufx-api runbook

How to run the HTTP server, configure it, rotate its keys, read what it
reports, and answer its alerts.

## Start

```sh
export NALUFX_API_KEY_SHA256="$(printf %s "$KEY" | sha256sum | cut -d' ' -f1)"
nalufx-api                     # listens on 127.0.0.1:8080
```

The server refuses to start without at least one key hash, and exits
with a message naming the first unusable setting.

Check it is up and ready:

```sh
curl -s localhost:8080/healthz   # {"status":"ok"} while the process runs
curl -s localhost:8080/readyz    # 200 when ready; 503 if the language model's endpoint does not answer
```

## Stop

Send SIGTERM (or Ctrl-C). The server stops accepting connections and
lets requests in flight finish for up to 25 seconds, then exits.
Give the process at least 30 seconds before a forced kill.

## Configure

Everything comes from the environment.

| Variable | Default | |
| --- | --- | --- |
| `NALUFX_ADDR` | `127.0.0.1:8080` | Listen address. |
| `NALUFX_API_KEY_SHA256` | (required) | Comma-separated hex SHA-256 hashes of the accepted keys. |
| `NALUFX_CORS_ORIGINS` | none | Comma-separated origins CORS allows; none denies them all. |
| `NALUFX_RATE_PER_MINUTE` | `600` | Requests per key per minute. |
| `NALUFX_REPORT_RATE_PER_MINUTE` | `30` | `/v1/report` calls per key per minute. |
| `NALUFX_TIMEOUT_SECS` | `30` | Time a request may take before it answers 408. |
| `NALUFX_CONCURRENCY` | `64` | Requests handled at once; the rest wait. |
| `NALUFX_LLM_PROVIDER`, `NALUFX_LLM_MODEL` | unset | The provider and model for `/v1/report`; without them it answers 503. |
| `NALUFX_LLM_MAX_TOKENS` | `2048` | The report's token limit. |
| `NALUFX_LOG_FORMAT` | `json` | `json` lines, or `text` for a terminal. |
| `RUST_LOG` | `info` | The log level, such as `warn` or `nalufx_api=debug`. |
| `NALUFX_AUDIT_LOG` | unset | A file to append the audit log to; unset, there is none. |

The provider's own key goes in its variable (`ANTHROPIC_API_KEY`,
`GEMINI_API_KEY`, `OPENAI_API_KEY`, ...); `nalufx providers` lists them.

## Rotate an API key

Keys are stored only as hashes, so a rotation never exposes one.

1. Give the client its new key, and add the new key's hash to
   `NALUFX_API_KEY_SHA256` beside the old one.
2. Restart the server (see Stop). Both keys now work.
3. When the client has switched, remove the old hash and restart again.

A key's id in the logs and the audit log is the first 8 hex characters
of its hash, so you can tell which key made a request without the key.
Each key has its own rate limit, so a new key starts with a full
allowance.

## Read the logs

Logs are written by RustLogs (`rlg`): one JSON object per event
(`NALUFX_LOG_FORMAT=text` gives logfmt). On Linux they go to journald
when the host has it (`journalctl`), and to standard output otherwise,
as in most containers; set `RLG_FALLBACK_STDOUT=1` to force standard
output.

Each line has `Level`, `Timestamp`, `Component` and `Description` (the
message), and under `Attributes` the fields of the request it belongs
to: `request_id` (the `x-request-id` the client sent, or one the server
made), `key_id`, `method`, `route`, `status`, `latency_ms`, plus
`span_path` (such as `request/allocate`), `trace_id` and `span_id`.
Every request ends with an `answered` line. A client quoting the
`x-request-id` header of a response, or the `request_id` of a problem,
leads straight to its lines.

The logs never hold an API key, a prompt, the investor's words or a
model's reply; tests check it. As a second line of defence, every line
and exported span is scrubbed of card numbers, JWTs, bearer tokens,
emails, IPv4 addresses, AWS key ids and language model provider keys
before it leaves the process.

## Export traces

Set `OTEL_EXPORTER_OTLP_ENDPOINT` to an OpenTelemetry Collector, such as
`http://localhost:4318`, and closed spans go to its `/v1/traces` as
OTLP/HTTP JSON, batched every 5 seconds or 512 spans. The `trace_id`
and `span_id` in the logs match the exported spans, so the collector
links the two. The endpoint must be plain `http://`: run the collector
beside the server and let it handle TLS to the backend.

| Variable | |
| --- | --- |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | The collector; `/v1/traces` is added. |
| `OTEL_EXPORTER_OTLP_TRACES_ENDPOINT` | The full traces URL, instead. |
| `OTEL_EXPORTER_OTLP_HEADERS` | `key=value` pairs, comma-separated, such as an auth header. |
| `OTEL_SERVICE_NAME` | The service name; `nalufx-api` by default. |
| `OTEL_SDK_DISABLED` | `true` turns export off. |

Each request is a `request` span (kind server, status error on a 5xx)
with child spans `forecast`, `allocate` or `report`, and `llm.attempt`
(kind client) for each call to a language model. On shutdown the server
waits up to 5 seconds for the last spans to be sent.

## Read the metrics

`GET /metrics` needs an API key, like the `/v1` routes. Scrape it with a
bearer token:

```yaml
scrape_configs:
  - job_name: nalufx-api
    authorization:
      credentials_file: /etc/prometheus/nalufx-key
    static_configs:
      - targets: ["nalufx-api:8080"]
```

| Metric | Labels | |
| --- | --- | --- |
| `http_requests_total` | route, status | Requests answered. |
| `http_request_duration_seconds` | route | Time to answer. |
| `allocation_duration_seconds` | method | Time the engine took to allocate. |
| `solver_failures_total` | method | Allocations whose solver failed. |
| `llm_requests_total` | provider, model, outcome | Report requests to a language model; `outcome` is `ok`, `timeout`, `rate_limited`, `provider_error`, `bad_reply`, `bad_input` or `not_configured`. |
| `llm_tokens_total` | provider, model, kind | Tokens used; `kind` is `prompt` or `completion`. |
| `llm_request_duration_seconds` | provider, model | Time a report request took. |

A route outside the served set counts as `other`.

## Alerts

`docs/alerts.yml` holds the three rules below, for Prometheus.

### NalufxHighErrorRate

More than 1% of requests answered 5xx for 10 minutes.

- Look at `http_requests_total` by route and status to see which route
  and which status.
- 503 on `/v1/allocate` or `/v1/report` with "the audit log could not
  be written": the audit log's disk is full or unwritable. Free space or
  fix permissions; requests recover at once.
- 502, 503 or 504 on `/v1/report`: the language model provider; see
  NalufxProviderErrors.
- Anything else: find the `request_id` of a failing request in the
  logs and read its lines.

### NalufxSlowRequests

The 95th percentile of forecast and allocate requests took over a
second for 10 minutes (reports wait on a language model and are left
out).

- `allocation_duration_seconds` by method shows whether the engine is
  the slow part. Allocations of up to 500 assets normally take under
  half a second.
- If the engine is fast, requests are waiting for a slot: raise
  `NALUFX_CONCURRENCY` if the machine has CPU to spare, or add replicas.

### NalufxProviderErrors

More than 5% of report requests to one provider and model failed over
15 minutes (bad input from callers is not counted).

- `llm_requests_total` by outcome says how: `timeout`, `rate_limited`
  (the provider's limit: lower `NALUFX_REPORT_RATE_PER_MINUTE` or raise
  the account's limit), `provider_error` (the provider's status page),
  or `bad_reply` (replies off the report schema: try a larger model).
- `/readyz` answers 503 while the provider's endpoint does not answer.

## The audit log

With `NALUFX_AUDIT_LOG` set, the server appends one JSON line per
allocation and per report. The file is only ever appended to; rotate it
by renaming it and restarting the server, which opens a new one.

An allocation's line holds its canonical input and that input's
SHA-256, the engine version, the method, its parameters and the
weights. To check a line, hash its `input` re-encoded in field order
(the server's own encoding) and compare with `input_sha256`; to
reproduce it, run the input through the same engine version: the
weights come out identical, bit for bit. A report's line holds the
SHA-256 of its input, the prompt's name and version, the provider, the
model and the tokens, never the investor's words.

If the file cannot be written, the request answers 503 rather than
send a result without its record.
