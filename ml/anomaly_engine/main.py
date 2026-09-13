"""Loop: train Isolation Forest, score assets, write health and alerts."""

from __future__ import annotations

import time
import traceback

from . import db
from .isolation_forest import group_rows_by_asset, score_latest, train_asset_model
from .predictive import contributing_metrics, failure_risks, health_percent, recommended_action

POLL_SECONDS = 10
RETRAIN_SECONDS = 300
TRAIN_WINDOW_MINUTES = 60
WARNING_SCORE = 0.70
CRITICAL_SCORE = 0.85


def _trend(previous: list[float], current: float) -> float:
    if len(previous) < 3:
        return 0.0
    older = sum(previous[1:4]) / min(3, len(previous) - 1)
    delta = current - older
    return max(0.0, min(1.0, delta / 0.35))


def _message(asset_id: str, score: float, contributing: list[str]) -> str:
    if score >= CRITICAL_SCORE:
        joined = ", ".join(contributing) if contributing else "multiple metrics"
        return f"Anomaly score {score:.2f} on {asset_id}. Potential equipment failure ({joined})."
    if score >= WARNING_SCORE:
        joined = ", ".join(contributing) if contributing else "telemetry drift"
        return f"Anomaly score {score:.2f} on {asset_id} ({joined})."
    return f"Normal anomaly score {score:.2f} on {asset_id}."


def evaluate_once(conn, models: dict) -> None:
    rows = db.fetch_recent_telemetry(conn, minutes=TRAIN_WINDOW_MINUTES)
    if not rows:
        print("No telemetry yet; waiting for ingestion.")
        return

    grouped = group_rows_by_asset(rows)
    for asset_id, asset_rows in grouped.items():
        site = asset_rows[-1].get("site") or "trondheim"
        model = models.get(asset_id)
        if model is None:
            model = train_asset_model(asset_rows)
            if model is None:
                continue
            models[asset_id] = model
            print(f"Trained Isolation Forest for {asset_id} ({len(asset_rows)} samples).")

        scored = score_latest(model, asset_rows)
        if scored is None:
            continue
        score, latest = scored
        contrib = contributing_metrics(latest, model.means, model.stds)
        previous = db.recent_scores(conn, asset_id)
        trend = _trend(previous, score)
        health = health_percent(score, trend)
        risk_7d, risk_30d, risk_90d = failure_risks(health, trend)
        action = recommended_action(contrib, score)
        is_anomaly = score >= WARNING_SCORE
        message = _message(asset_id, score, contrib)

        db.insert_anomaly(
            conn,
            site=site,
            asset_id=asset_id,
            score=score,
            is_anomaly=is_anomaly,
            contributing=contrib,
            message=message,
        )
        db.insert_health(
            conn,
            site=site,
            asset_id=asset_id,
            health=health,
            risk_7d=risk_7d,
            risk_30d=risk_30d,
            risk_90d=risk_90d,
            action=action,
        )

        if score >= CRITICAL_SCORE:
            inserted = db.insert_alert_if_new(
                conn,
                site=site,
                asset_id=asset_id,
                severity="CRITICAL",
                alert_type="anomaly_critical",
                message=message,
            )
            if inserted:
                print(f"CRITICAL anomaly alert for {asset_id}: {score:.2f}")
        elif score >= WARNING_SCORE:
            inserted = db.insert_alert_if_new(
                conn,
                site=site,
                asset_id=asset_id,
                severity="WARNING",
                alert_type="anomaly",
                message=message,
            )
            if inserted:
                print(f"WARNING anomaly alert for {asset_id}: {score:.2f}")
        else:
            print(f"{asset_id}: score={score:.2f} health={health:.0f}%")


def run() -> None:
    print("Starting Industrial Intelligence Anomaly Engine...")
    print(f"Database: {db.database_url()}")

    while True:
        try:
            conn = db.connect()
            db.ensure_schema(conn)
            print("Connected to TimescaleDB.")
            break
        except Exception as error:
            print(f"Waiting for TimescaleDB: {error}")
            time.sleep(3)

    models: dict = {}
    last_train = 0.0

    while True:
        try:
            now = time.time()
            if now - last_train >= RETRAIN_SECONDS:
                models.clear()
                last_train = now
                print("Retraining Isolation Forest models.")
            evaluate_once(conn, models)
        except Exception:
            traceback.print_exc()
            try:
                conn.close()
            except Exception:
                pass
            time.sleep(3)
            try:
                conn = db.connect()
                db.ensure_schema(conn)
            except Exception as error:
                print(f"Reconnect failed: {error}")
        time.sleep(POLL_SECONDS)
