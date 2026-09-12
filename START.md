# Oppstart – Industrial Intelligence

## Hurtigstart (anbefalt)

Fra repo-roten:

```powershell
.\start-all.ps1
```

Starter MQTT, TimescaleDB, ingestion, factory-simulator, alerts, backend og frontend i egne vinduer.

Stopp:

```powershell
.\stop-all.ps1
```

Dashboard: http://localhost:3001

**Krav:** Docker Desktop kjører. Lokal Windows-Postgres på port `5432` må være stoppet (admin-PowerShell: `Stop-Service postgresql-x64-17`).

---

## Manuell oppstart

Kjør tingene i denne rekkefølgen. Hver del må være oppe før neste.

```text
Docker Desktop
    → MQTT (Mosquitto)
    → TimescaleDB
    → Ingestion-service
    → Factory-simulator
    → Alarm Engine
    → OT Security Monitor
    → Backend
    → Frontend
```

---

## Forutsetninger

- Docker Desktop kjører
- Rust / Cargo er installert
- Node.js / npm er installert (frontend)
- Lokal Windows-Postgres på port `5432` er **stopp**et (ellers treffer ingestion feil database)

```powershell
Get-Service *postgres*
Stop-Service postgresql-x64-17   # bruk eksakt navn fra lista (krever admin)
```

Eller med Docker Compose:

```powershell
docker compose up -d
```

---

## 1. MQTT-broker (Mosquitto)

```powershell
docker compose up -d mqtt
```

Alternativ uten Compose:

```powershell
docker rm -f industrial-mqtt 2>$null
docker run -d --name industrial-mqtt -p 1883:1883 eclipse-mosquitto:2 mosquitto -c /mosquitto-no-auth.conf
```

Sjekk:

```powershell
docker ps --filter name=industrial-mqtt
```

---

## 2. TimescaleDB

```powershell
docker compose up -d timescaledb
```

Første gang opprettes tabeller automatisk via `infrastructure/docker/init.sql`.

Alternativ uten Compose:

```powershell
docker rm -f industrial-timescaledb 2>$null
docker run -d --name industrial-timescaledb `
  -e POSTGRES_USER=industrial `
  -e POSTGRES_PASSWORD=industrial123 `
  -e POSTGRES_DB=industrial_intelligence `
  -p 5432:5432 `
  timescale/timescaledb:latest-pg17
```

Sjekk:

```powershell
docker ps --filter name=industrial-timescaledb
```

### Manuell tabell-opprettelse (kun hvis volume allerede finnes uten schema)

```powershell
docker exec -it industrial-timescaledb psql -U industrial -d industrial_intelligence
```

I `psql`:

```sql
CREATE EXTENSION IF NOT EXISTS timescaledb;

CREATE TABLE IF NOT EXISTS telemetry (
    time        TIMESTAMPTZ NOT NULL,
    site        TEXT NOT NULL,
    asset_type  TEXT NOT NULL,
    asset_id    TEXT NOT NULL,
    metric      TEXT NOT NULL,
    value       DOUBLE PRECISION NOT NULL
);

SELECT create_hypertable('telemetry', 'time', if_not_exists => TRUE);
```

Avslutt med `\q`.

---

## 3. Ingestion-service

Abonnerer på MQTT og skriver til TimescaleDB.

```powershell
cd services\ingestion
cargo run
```

Forventet output:

```text
Starting Industrial Intelligence Ingestion Service...
Connected to TimescaleDB.
Subscribed to factory telemetry.
```

La denne terminalen stå åpen.

---

## 4. Factory-simulator

Publiserer telemetri til MQTT hvert sekund.

```powershell
cd simulators\factory-simulator
cargo run
```

Forventet output:

```text
Published factory telemetry
Published factory telemetry
...
```

La denne terminalen stå åpen.

---

## 5. Verifiser data

```powershell
docker exec -it industrial-timescaledb psql -U industrial -d industrial_intelligence
```

```sql
SELECT * FROM telemetry ORDER BY time DESC LIMIT 20;
```

---

## 6. Alarm Engine

Sjekker telemetri mot terskler og skriver til `alerts`.

```powershell
cd services\alerts
cargo run
```

Regler:

| Metric | WARNING | CRITICAL |
| ------ | ------- | -------- |
| temperature | > 90 °C | > 110 °C |
| vibration | > 5 mm/s | > 9 mm/s |
| pressure | < 3.5 bar | < 2.5 bar |
| RPM avvik | > 20 % | > 30 % |

Forventet output ved brudd:

```text
--------------------------------
CRITICAL

P101

High vibration detected on pump P101

9.2 mm/s

Detected:
20:42:31
--------------------------------
```

Verifiser:

```sql
SELECT * FROM alerts ORDER BY time DESC LIMIT 10;
```

---

## 7. OT Security Monitor

Fanger nettverkstrafikk (Npc), parser Modbus TCP / OPC UA, lagrer i `network_events`.

```powershell
cd services\security-monitor

# Liste interfaces
cargo run -- --list

# Demo (anbefalt uten ekte OT-trafikk)
cargo run -- --demo

# Live capture (kan kreve admin)
cargo run -- --iface "\Device\NPF_Loopback"
```

Mistenkelig Modbus write fra ukjent IP oppretter også `alerts` + `incidents`.

---

## 8. Backend + Frontend

```powershell
cd apps\backend
cargo run
```

```powershell
cd apps\frontend
npm run dev
```

Åpne http://localhost:3001

Backend må kjøre på port 3000 (frontend henter data derfra).

---

## Hurtigstart (containere allerede opprettet)

Hvis `industrial-mqtt` og `industrial-timescaledb` finnes fra før:

```powershell
docker start industrial-mqtt industrial-timescaledb
```

Eller:

```powershell
docker compose start
```

Deretter, i separate terminaler — eller bare `.\start-all.ps1`.

---

## Stopp

`Ctrl+C` i terminalene, eller:

```powershell
.\stop-all.ps1
```

```powershell
docker compose stop
```

---

## Porte og credentials

| Tjeneste     | Host        | Port |
| ------------ | ----------- | ---- |
| Frontend     | `localhost` | 3001 |
| Backend      | `localhost` | 3000 |
| MQTT         | `localhost` | 1883 |
| TimescaleDB  | `localhost` | 5432 |

| Felt     | Verdi                     |
| -------- | ------------------------- |
| User     | `industrial`              |
| Password | `industrial123`           |
| Database | `industrial_intelligence` |

Connection string:

```text
postgres://industrial:industrial123@localhost:5432/industrial_intelligence
```

MQTT-topic:

```text
factory/trondheim/+/+/telemetry
```
