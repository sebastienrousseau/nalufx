"""scikit-learn's Ledoit-Wolf on a deterministic 50 x 5 returns matrix.

The reference for covariance::tests::ledoit_wolf_matches_scikit_learn.
The matrix comes from a 64-bit LCG whose values are exact dyadic
fractions, so the Rust test rebuilds it bit for bit.

Usage: uv run --no-project --with scikit-learn==1.7.2 python ledoit_wolf_reference.py
"""
import numpy as np
import sklearn
from sklearn.covariance import LedoitWolf

MASK = (1 << 64) - 1
state = 20261127


def uniform():
    global state
    state = (state * 6364136223846793005 + 1442695040888963407) & MASK
    return (state >> 11) / float(1 << 53) - 0.5


rows = []
for _ in range(50):
    market = uniform()
    rows.append([0.02 * (0.3 + 0.3 * j) * market + 0.01 * uniform() for j in range(5)])
x = np.array(rows)
lw = LedoitWolf().fit(x)
print("sklearn", sklearn.__version__, "numpy", np.__version__)
print("shrinkage", repr(float(lw.shrinkage_)))
print("diag", [repr(float(v)) for v in np.diag(lw.covariance_)])
print("row0", [repr(float(v)) for v in lw.covariance_[0]])
print("x00", repr(float(x[0, 0])), "x49_4", repr(float(x[49, 4])))
