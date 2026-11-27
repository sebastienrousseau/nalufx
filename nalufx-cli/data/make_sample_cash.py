#!/usr/bin/env python3
"""Writes sample-cash.csv: 120 days of synthetic daily net cash flows with
a 5-business-day cycle (payroll out on day 5, receipts in on day 2), a
slow upward trend and noise. Deterministic (fixed-seed LCG). Synthetic:
no real company's figures.

Usage: python3 make_sample_cash.py > sample-cash.csv
"""
import math

state = 20261127
CYCLE = [0.0, 120.0, 40.0, -30.0, -180.0]


def uniform():
    global state
    state = (1664525 * state + 1013904223) % 2**32
    return state / 2**32 - 0.5


print("day,net_cash_flow")
for day in range(120):
    value = 1000.0 + 1.5 * day + CYCLE[day % 5] + 25.0 * uniform()
    print(f"{day + 1},{value:.2f}")
