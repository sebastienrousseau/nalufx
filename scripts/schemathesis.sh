#!/usr/bin/env bash
# Runs Schemathesis against nalufx-api's committed openapi.json, on a
# release build backed by the stub language model, and fails on any 5xx.
# CI runs it; locally: scripts/schemathesis.sh (needs uv).
set -euo pipefail
cd "$(dirname "$0")/.."

KEY="schemathesis-key"
API_PORT=18088
STUB_PORT=18099
SCHEMATHESIS="schemathesis==4.33.4"

cargo build --release -p nalufx-api --bin nalufx-api
python3 scripts/stub-llm.py "$STUB_PORT" &
stub=$!
NALUFX_API_KEY_SHA256="$(printf %s "$KEY" | sha256sum | cut -d' ' -f1)" \
NALUFX_ADDR="127.0.0.1:${API_PORT}" \
NALUFX_LLM_PROVIDER=openai_compatible NALUFX_LLM_MODEL=stub \
NALUFX_LLM_BASE_URL="http://127.0.0.1:${STUB_PORT}/v1" \
NALUFX_RATE_PER_MINUTE=100000 NALUFX_REPORT_RATE_PER_MINUTE=100000 \
  target/release/nalufx-api &
api=$!
trap 'kill "$api" "$stub" 2>/dev/null || true' EXIT

for _ in $(seq 1 50); do
  curl -fsS "http://127.0.0.1:${API_PORT}/readyz" >/dev/null 2>&1 && break
  sleep 0.2
done
curl -fsS "http://127.0.0.1:${API_PORT}/readyz" >/dev/null

if command -v uvx >/dev/null; then
  run=(uvx --from "$SCHEMATHESIS" schemathesis)
else
  run=(pipx run --spec "$SCHEMATHESIS" schemathesis)
fi
"${run[@]}" run nalufx-api/openapi.json \
  --url "http://127.0.0.1:${API_PORT}" \
  --header "Authorization: Bearer ${KEY}" \
  --checks not_a_server_error \
  --max-examples 50
