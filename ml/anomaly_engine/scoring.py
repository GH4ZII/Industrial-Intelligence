"""Map Isolation Forest outputs to a 0–1 anomaly score."""

from __future__ import annotations


def map_decision_to_score(decision: float) -> float:
    """Convert IsolationForest.decision_function to 0–1 (higher = more anomalous).

    Inliers typically have positive decisions (~0.1–0.4 → scores near 0.08).
    Outliers have negative decisions (→ scores near 0.9).
    """
    return max(0.0, min(1.0, 0.5 - decision))


def zscore_anomaly(value: float, mean: float, std: float) -> float:
    """Fallback 0–1 score from a single z-score when the model cannot train."""
    z = abs(value - mean) / (std + 1e-9)
    return max(0.0, min(1.0, z / 6.0))
