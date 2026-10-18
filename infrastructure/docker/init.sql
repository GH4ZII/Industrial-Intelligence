-- Runs only on first container init (empty volume).
CREATE EXTENSION IF NOT EXISTS timescaledb;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS telemetry (
    time        TIMESTAMPTZ NOT NULL,
    site        TEXT NOT NULL,
    asset_type  TEXT NOT NULL,
    asset_id    TEXT NOT NULL,
    metric      TEXT NOT NULL,
    value       DOUBLE PRECISION NOT NULL
);

SELECT create_hypertable('telemetry', 'time', if_not_exists => TRUE);

CREATE TABLE IF NOT EXISTS roles (
    id   SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

INSERT INTO roles (name) VALUES ('admin'), ('operator')
ON CONFLICT (name) DO NOTHING;

CREATE TABLE IF NOT EXISTS users (
    id            BIGSERIAL PRIMARY KEY,
    email         TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role_id       INTEGER NOT NULL REFERENCES roles(id),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS sessions (
    token      UUID PRIMARY KEY,
    user_id    BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE IF NOT EXISTS alerts (
    id            BIGSERIAL PRIMARY KEY,
    time          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    site          TEXT NOT NULL,
    asset_id      TEXT NOT NULL,
    severity      TEXT NOT NULL,
    alert_type    TEXT NOT NULL,
    message       TEXT NOT NULL,
    acknowledged  BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX IF NOT EXISTS idx_alerts_asset ON alerts (asset_id);

CREATE TABLE IF NOT EXISTS incidents (
    id          BIGSERIAL PRIMARY KEY,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    severity    TEXT NOT NULL,
    status      TEXT NOT NULL,
    title       TEXT NOT NULL,
    asset_id    TEXT,
    description TEXT,
    assigned_to TEXT,
    mitre_id    TEXT,
    mitre_name  TEXT
);

CREATE INDEX IF NOT EXISTS idx_incidents_created ON incidents (created_at DESC);

CREATE TABLE IF NOT EXISTS incident_events (
    id          BIGSERIAL PRIMARY KEY,
    incident_id BIGINT NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
    time        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    message     TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_incident_events_incident
    ON incident_events (incident_id, time);

CREATE TABLE IF NOT EXISTS anomaly_scores (
    time                   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    site                   TEXT NOT NULL,
    asset_id               TEXT NOT NULL,
    score                  DOUBLE PRECISION NOT NULL,
    is_anomaly             BOOLEAN NOT NULL DEFAULT FALSE,
    contributing_metrics   TEXT,
    message                TEXT
);

SELECT create_hypertable('anomaly_scores', 'time', if_not_exists => TRUE);

CREATE INDEX IF NOT EXISTS idx_anomaly_scores_asset
    ON anomaly_scores (asset_id, time DESC);

CREATE TABLE IF NOT EXISTS asset_health (
    time                 TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    site                 TEXT NOT NULL,
    asset_id             TEXT NOT NULL,
    health_percent       DOUBLE PRECISION NOT NULL,
    risk_7d              DOUBLE PRECISION NOT NULL,
    risk_30d             DOUBLE PRECISION NOT NULL,
    risk_90d             DOUBLE PRECISION NOT NULL,
    recommended_action   TEXT
);

CREATE INDEX IF NOT EXISTS idx_asset_health_asset
    ON asset_health (asset_id, time DESC);

CREATE TABLE IF NOT EXISTS network_events (
    id                   BIGSERIAL PRIMARY KEY,
    time                 TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    source_ip            TEXT NOT NULL,
    destination_ip       TEXT NOT NULL,
    source_port          INTEGER NOT NULL,
    destination_port     INTEGER NOT NULL,
    transport_protocol   TEXT NOT NULL,
    application_protocol TEXT,
    payload_size         INTEGER NOT NULL,
    modbus_function      TEXT,
    opcua_message        TEXT,
    suspicious           BOOLEAN NOT NULL DEFAULT FALSE,
    severity             TEXT,
    message              TEXT
);

CREATE INDEX IF NOT EXISTS idx_network_events_time ON network_events (time DESC);
