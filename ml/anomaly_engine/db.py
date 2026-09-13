"""TimescaleDB access for the anomaly engine."""

from __future__ import annotations

import json
import os
from datetime import datetime, timezone

import psycopg2
from psycopg2.extras import RealDictCursor

DEFAULT_DATABASE_URL = (
    "postgres://industrial:industrial123@localhost:5432/industrial_intelligence"
)


def database_url() -> str:
    return os.environ.get("DATABASE_URL", DEFAULT_DATABASE_URL)


def connect():
    return psycopg2.connect(database_url())


def ensure_schema(conn) -> None:
    statements = [
        """
        CREATE TABLE IF NOT EXISTS anomaly_scores (
            time                   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            site                   TEXT NOT NULL,
            asset_id               TEXT NOT NULL,
            score                  DOUBLE PRECISION NOT NULL,
            is_anomaly             BOOLEAN NOT NULL DEFAULT FALSE,
            contributing_metrics   TEXT,
            message                TEXT
        )
        """,
        """
        CREATE INDEX IF NOT EXISTS idx_anomaly_scores_asset
            ON anomaly_scores (asset_id, time DESC)
        """,
        """
        CREATE TABLE IF NOT EXISTS asset_health (
            time                 TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            site                 TEXT NOT NULL,
            asset_id             TEXT NOT NULL,
            health_percent       DOUBLE PRECISION NOT NULL,
            risk_7d              DOUBLE PRECISION NOT NULL,
            risk_30d             DOUBLE PRECISION NOT NULL,
            risk_90d             DOUBLE PRECISION NOT NULL,
            recommended_action   TEXT
        )
        """,
        """
        CREATE INDEX IF NOT EXISTS idx_asset_health_asset
            ON asset_health (asset_id, time DESC)
        """,
        """
        CREATE TABLE IF NOT EXISTS alerts (
            id            BIGSERIAL PRIMARY KEY,
            time          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            site          TEXT NOT NULL,
            asset_id      TEXT NOT NULL,
            severity      TEXT NOT NULL,
            alert_type    TEXT NOT NULL,
            message       TEXT NOT NULL,
            acknowledged  BOOLEAN NOT NULL DEFAULT FALSE
        )
        """,
    ]
    with conn.cursor() as cur:
        for sql in statements:
            cur.execute(sql)
        try:
            cur.execute(
                "SELECT create_hypertable('anomaly_scores', 'time', if_not_exists => TRUE)"
            )
        except Exception:
            conn.rollback()
            for sql in statements:
                cur.execute(sql)
    conn.commit()


def fetch_recent_telemetry(conn, minutes: int = 60) -> list[dict]:
    with conn.cursor(cursor_factory=RealDictCursor) as cur:
        cur.execute(
            """
            SELECT time, site, asset_type, asset_id, metric, value
            FROM telemetry
            WHERE time > NOW() - (%s * INTERVAL '1 minute')
            ORDER BY asset_id, time ASC
            """,
            (minutes,),
        )
        rows = cur.fetchall()
    return [dict(row) for row in rows]


def insert_anomaly(
    conn,
    *,
    site: str,
    asset_id: str,
    score: float,
    is_anomaly: bool,
    contributing: list[str],
    message: str,
) -> None:
    with conn.cursor() as cur:
        cur.execute(
            """
            INSERT INTO anomaly_scores (
                time, site, asset_id, score, is_anomaly, contributing_metrics, message
            )
            VALUES (NOW(), %s, %s, %s, %s, %s, %s)
            """,
            (
                site,
                asset_id,
                score,
                is_anomaly,
                json.dumps(contributing),
                message,
            ),
        )
    conn.commit()


def insert_health(
    conn,
    *,
    site: str,
    asset_id: str,
    health: float,
    risk_7d: float,
    risk_30d: float,
    risk_90d: float,
    action: str,
) -> None:
    with conn.cursor() as cur:
        cur.execute(
            """
            INSERT INTO asset_health (
                time, site, asset_id, health_percent,
                risk_7d, risk_30d, risk_90d, recommended_action
            )
            VALUES (NOW(), %s, %s, %s, %s, %s, %s, %s)
            """,
            (site, asset_id, health, risk_7d, risk_30d, risk_90d, action),
        )
    conn.commit()


def insert_alert_if_new(
    conn,
    *,
    site: str,
    asset_id: str,
    severity: str,
    alert_type: str,
    message: str,
) -> bool:
    with conn.cursor() as cur:
        cur.execute(
            """
            SELECT COUNT(*)::BIGINT
            FROM alerts
            WHERE asset_id = %s
              AND alert_type = %s
              AND acknowledged = false
              AND time > NOW() - INTERVAL '5 minutes'
            """,
            (asset_id, alert_type),
        )
        existing = cur.fetchone()[0]
        if existing > 0:
            conn.commit()
            return False
        cur.execute(
            """
            INSERT INTO alerts (
                time, site, asset_id, severity, alert_type, message, acknowledged
            )
            VALUES (NOW(), %s, %s, %s, %s, %s, false)
            """,
            (site, asset_id, severity, alert_type, message),
        )
    conn.commit()
    return True


def recent_scores(conn, asset_id: str, limit: int = 12) -> list[float]:
    with conn.cursor() as cur:
        cur.execute(
            """
            SELECT score
            FROM anomaly_scores
            WHERE asset_id = %s
            ORDER BY time DESC
            LIMIT %s
            """,
            (asset_id, limit),
        )
        rows = cur.fetchall()
    return [float(row[0]) for row in rows]


def utcnow() -> datetime:
    return datetime.now(timezone.utc)
