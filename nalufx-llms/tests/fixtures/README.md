# Recorded provider responses

Each `*_success.json` is the shape of a successful reply from that
provider's API, as its documentation gives it, with the text and token
counts set to the values the contract tests assert. No test calls a real
provider; the nightly `llm-live` workflow does, when its secrets are set.
