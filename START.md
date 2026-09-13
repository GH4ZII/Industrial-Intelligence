# Oppstart – Industrial Intelligence

To måter å kjøre stakken på: **Docker Compose** (hele plattformen) eller **Windows-dev** med `start-all.ps1`.

## Docker Compose (anbefalt)

Fra repo-roten. Krever Docker.

```bash
docker compose up --build
```

| Tjeneste | URL |
| --- | --- |
| Dashboard | http://localhost:3001 |
| Backend | http://localhost:3000 |
| Grafana | http://localhost:3002 (admin / industrial) |
| Prometheus | http://localhost:9090 |
| MQTT | localhost:1883 |
| TimescaleDB | localhost:5432 |

Stopp:

```bash
docker compose down
```

Containere: factory-simulator, mqtt, ingestion, alerts, security-monitor (--demo), ml-service, backend, frontend, timescaledb, prometheus, grafana.

---

## Hurtigstart (Windows-dev)

Fra repo-roten:

```powershell
.\start-all.ps1
```

Starter MQTT, TimescaleDB, ingestion, factory-simulator, alerts, security-monitor, anomaly-engine, backend og frontend i egne vinduer.

Stopp:

```powershell
.\stop-all.ps1
```

Dashboard: http://localhost:3001

**Krav:** Docker Desktop, Rust/Cargo, Node.js/npm, Python 3.11+. Lokal Windows-Postgres på port `5432` må være stoppet (admin-PowerShell: `Stop-Service postgresql-x64-17`).

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
    → Anomaly Engine
    → Backend
    → Frontend
```

---

## Forutsetninger

- Docker Desktop kjører
- Rust / Cargo er installert
- Node.js / npm er installert (frontend)
- Python 3.11+ (anomaly engine)
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

Sjekk:

```powershell
docker ps --filter name=industrial-timescaledb
```

Connection string (overstyr med `DATABASE_URL`):

```text
postgres://industrial:industrial123@localhost:5432/industrial_intelligence
```

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
...
```

MQTT-topic:

```text
factory/trondheim/+/+/telemetry
```

---

## 5. Alarm Engine

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

---

## 6. OT Security Monitor

Fanger nettverkstrafikk (Npcap), parser Modbus TCP / OPC UA, lagrer i `network_events`. Mistenkelig Modbus write får MITRE **T0831** og åpner incident med tidslinje.

```powershell
cd services\security-monitor

# Liste interfaces
cargo run -- --list

# Demo (anbefalt uten ekte OT-trafikk)
cargo run -- --demo

# Live capture (kan kreve admin)
cargo run -- --iface "\Device\NPF_Loopback"
```

---

## 7. Anomaly Engine

Isolation Forest + predictive maintenance. Skriver `anomaly_scores`, `asset_health` og anomaly-alerts.

```powershell
cd ml
python -m venv .venv
.\.venv\Scripts\pip install -r requirements.txt
.\.venv\Scripts\python -m anomaly_engine
```

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

- Dashboard: helse, alarmer, incidents
- Twin: `/twin`
- Incidents: `/incidents`
- Asset: grafer, anomaly score, 7/30/90-dagers risiko

Backend må kjøre på port 3000 (frontend henter data derfra, `NEXT_PUBLIC_API_URL`).

---

## Porte og credentials

| Tjeneste     | Host        | Port |
| ------------ | ----------- | ---- |
| Frontend     | `localhost` | 3001 |
| Backend      | `localhost` | 3000 |
| Grafana      | `localhost` | 3002 |
| Prometheus   | `localhost` | 9090 |
| MQTT         | `localhost` | 1883 |
| TimescaleDB  | `localhost` | 5432 |

| Felt     | Verdi                     |
| -------- | ------------------------- |
| User     | `industrial`              |
| Password | `industrial123`           |
| Database | `industrial_intelligence` |
