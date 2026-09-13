"""Map Isolation Forest outputs to a 0–1 anomaly score."""

from __future__ import annotations

import math


def map_decision_to_score(decision: float) -> float:
    """Convert IsolationForest.decision_function to 0–1 (higher = more anomalous).

    Positive decisions are inliers; negative decisions are outliers.
    """
    score = 1.0 / (1.0 + math.exp(decision * 5.0))
    return max(0.0, min(1.0, score))


def zscore_anomaly(value: float, mean: float, std: float) -> float:
    """Fallback 0–1 score from a single z-score when the model cannot train."""
    z = abs(value - mean) / (std + 1e-9)
    return max(0.0, min(1.0, z / 6.0))
