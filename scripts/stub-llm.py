#!/usr/bin/env python3
"""A stand-in language model for API tests: an OpenAI-compatible
`POST /v1/chat/completions` that always answers with a report matching
the nalufx allocation_report schema, and a `GET` that answers 200 so
the server's readiness probe passes. Never used in production.

Usage: scripts/stub-llm.py PORT
"""
import http.server
import json
import sys

REPORT = {
    "summary": "A stub report.",
    "rationale": ["The figures were supplied by the caller."],
    "risks": ["This is a test double."],
    "disclaimer": "For testing only; not investment advice.",
}
REPLY = json.dumps({
    "choices": [{"message": {"content": json.dumps(REPORT)}}],
    "model": "stub",
    "usage": {"prompt_tokens": 1, "completion_tokens": 1},
}).encode()


class Stub(http.server.BaseHTTPRequestHandler):
    def _send(self, body):
        self.send_response(200)
        self.send_header("content-type", "application/json")
        self.send_header("content-length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        self._send(b"{}")

    def do_POST(self):
        self.rfile.read(int(self.headers.get("content-length", 0)))
        self._send(REPLY)

    def log_message(self, *args):
        pass


http.server.ThreadingHTTPServer(("127.0.0.1", int(sys.argv[1])), Stub).serve_forever()
