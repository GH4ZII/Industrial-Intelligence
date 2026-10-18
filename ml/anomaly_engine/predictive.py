"""Predictive maintenance: health, failure risk, recommended action."""

from __future__ import annotations


def clamp(value: float, low: float, high: float) -> float:
    return max(low, min(high, value))


def health_percent(score: float, trend: float = 0.0) -> float:
    """Invert anomaly score and penalise a rising trend."""
    base = 100.0 * (1.0 - score)
    penalty = clamp(trend, 0.0, 1.0) * 12.0
    return round(clamp(base - penalty, 0.0, 100.0), 1)


def failure_risks(health: float, trend: float = 0.0) -> tuple[float, float, float]:
    """Heuristic 7/30/90-day failure probabilities from health and trend."""
    gap = 100.0 - health
    bump = clamp(trend, 0.0, 1.0)
    risk_7d = round(clamp(gap * 0.18 + bump * 4.0, 0.0, 95.0), 1)
    risk_30d = round(clamp(gap * 0.45 + bump * 10.0, 0.0, 95.0), 1)
    risk_90d = round(clamp(gap * 0.85 + bump * 16.0, 0.0, 99.0), 1)
    return risk_7d, risk_30d, risk_90d


def recommended_action(contributing: list[str], score: float) -> str:
    if score < 0.40:
        return "No action required. Continue routine monitoring."

    metrics = {m.lower() for m in contributing}

    if "vibration" in metrics:
        return "Inspect bearing within 30 days."
    if "temperature" in metrics:
        return "Inspect cooling and lubrication within 14 days."
    if "pressure" in metrics:
        return "Inspect seals and suction line within 14 days."
    if "rpm" in metrics:
        return "Inspect coupling and load within 30 days."
    if "current" in metrics or "power" in metrics:
        return "Inspect motor windings and supply within 30 days."
    if "flow" in metrics or "position" in metrics:
        return "Inspect valve actuator within 30 days."
    if "level" in metrics:
        return "Inspect tank instrumentation within 30 days."
    if score >= 0.85:
        return "Potential equipment failure. Inspect asset immediately."
    return "Inspect equipment within 30 days."


def contributing_metrics(
    latest: dict[str, float],
    means: dict[str, float],
    stds: dict[str, float],
    limit: int = 3,
) -> list[str]:
    ranked: list[tuple[str, float]] = []
    for metric, value in latest.items():
        std = stds.get(metric, 0.0)
        mean = means.get(metric, value)
        z = abs(value - mean) / (std + 1e-9)
        if z > 1.5:
            ranked.append((metric, z))
    ranked.sort(key=lambda item: item[1], reverse=True)
    return [name for name, _ in ranked[:limit]]
