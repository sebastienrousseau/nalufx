#!/usr/bin/env python3
"""Writes assets_500.csv: 500 synthetic daily returns of 5 assets.

Deterministic (a fixed-seed linear congruential generator, no library
randomness), so it regenerates byte for byte. Each asset is a one-factor
model, r = drift + beta * market + idiosyncratic noise, with betas and
volatilities spanning a bond-like asset to a volatile equity. Synthetic:
no real prices.

Usage: python3 make_assets_500.py > assets_500.csv
"""
import math

state = 500_2026_11_27
ASSETS = [
    # name,    drift,    beta, idiosyncratic volatility
    ("BOND",   0.00010,  0.05, 0.0020),
    ("CREDIT", 0.00015,  0.30, 0.0040),
    ("EQUITY", 0.00035,  1.00, 0.0060),
    ("SMALL",  0.00040,  1.30, 0.0110),
    ("GOLD",   0.00020, -0.10, 0.0090),
]


def uniform():
    global state
    state = (1664525 * state + 1013904223) % 2**32
    return state / 2**32


def normal():
    u1, u2 = uniform(), uniform()
    return math.sqrt(-2.0 * math.log(1.0 - u1)) * math.cos(2.0 * math.pi * u2)


print(",".join(name for name, *_ in ASSETS))
for day in range(500):
    market = 0.009 * normal()
    row = [drift + beta * market + vol * normal() for _, drift, beta, vol in ASSETS]
    print(",".join(f"{r:.8f}" for r in row))
