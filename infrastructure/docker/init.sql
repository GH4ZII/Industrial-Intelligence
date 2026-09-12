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
    severity    TEXT NOT NULL,
    status      TEXT NOT NULL,
    title       TEXT NOT NULL,
    asset_id    TEXT,
    description TEXT
);

CREATE INDEX IF NOT EXISTS idx_incidents_created ON incidents (created_at DESC);
