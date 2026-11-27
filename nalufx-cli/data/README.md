# Sample data

Synthetic, for trying the `nalufx` command without a network connection.
No file here describes a real asset or company.

| File | What | Made by |
| --- | --- | --- |
| `sample-returns.csv` | 500 daily returns of 5 assets (BOND, CREDIT, EQUITY, SMALL, GOLD), one-factor model | `nalufx-engine/tests/fixtures/make_assets_500.py` |
| `sample-cash.csv` | 120 days of daily net cash flows with a 5-day cycle | `make_sample_cash.py` |

```shell
nalufx allocate --file nalufx-cli/data/sample-returns.csv --method risk-parity
nalufx forecast --file nalufx-cli/data/sample-cash.csv --column net_cash_flow --seasonal 5
nalufx backtest --file nalufx-cli/data/sample-returns.csv --method all
```
