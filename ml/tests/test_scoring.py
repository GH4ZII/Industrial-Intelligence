from anomaly_engine.predictive import (
    contributing_metrics,
    failure_risks,
    health_percent,
    recommended_action,
)
from anomaly_engine.scoring import map_decision_to_score, zscore_anomaly


def test_inlier_decision_maps_low():
    assert map_decision_to_score(1.0) < 0.20


def test_outlier_decision_maps_high():
    assert map_decision_to_score(-1.0) > 0.80


def test_zscore_caps_at_one():
    assert zscore_anomaly(100, 0, 1) == 1.0
    assert zscore_anomaly(0, 0, 1) == 0.0


def test_health_inverts_score():
    assert health_percent(0.08) > 90
    assert health_percent(0.94) < 10


def test_risk_increases_with_poor_health():
    low = failure_risks(82.0, 0.0)
    high = failure_risks(20.0, 0.8)
    assert low[0] < high[0]
    assert low[2] < high[2]


def test_bearing_recommendation():
    action = recommended_action(["vibration", "temperature"], 0.8)
    assert "bearing" in action.lower()


def test_contributing_metrics_picks_outliers():
    latest = {"temperature": 110.0, "vibration": 2.0, "rpm": 1450.0}
    means = {"temperature": 63.0, "vibration": 2.1, "rpm": 1450.0}
    stds = {"temperature": 2.0, "vibration": 0.4, "rpm": 10.0}
    contrib = contributing_metrics(latest, means, stds)
    assert contrib[0] == "temperature"
