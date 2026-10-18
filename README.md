# Industrial Intelligence

Overvåkingsplattform for industri og OT-miljøer. Den samler sanntidsdata fra maskiner, viser anleggshelse i et dashboard, lager alarmer ved unormal drift, og overvåker samtidig OT-nettverket for mistenkelig aktivitet mot PLS-er.

Målet er programvare som kunne brukes i norsk industri, energi, olje/gass, maritim sektor eller kritisk infrastruktur — der produksjon og cybersikkerhet må ses sammen.

---

- Innhenter telemetri via MQTT, Modbus TCP og OPC UA
- Prosesserer sanntids- og historiske sensordata
- Viser plant health, digital twin og asset-status
- Genererer alarmer ved terskelbrudd og ML-anomalier
- Estimerer maskinhelse og vedlikeholdsrisiko
- Overvåker OT-nettverk for mistenkelig aktivitet (MITRE ATT&CK for ICS)
- Støtter incident management og predictive maintenance

## Arkitektur

Tjenestene er løst koblet. Ingest, alarmer og pakkeanalyse er egne prosesser som kommuniserer via MQTT og databasen.

```text
Virtuell fabrikk (simulator)
        │  MQTT hvert sekund
        ▼
   Mosquitto (broker)
        │
        ▼
 Industrial Gateway → Message Broker (MQTT)
        │
        ├─ Data Processing → Time-Series DB
        ├─ Anomaly Engine (Isolation Forest)
        └─ Security Monitor → Alert Engine
                    │
              Backend API → Dashboard
                    │
              Prometheus / Grafana
```

**Simulator** publiserer JSON på topics som `factory/trondheim/pump/P101/telemetry`.

**Ingestion** abonnerer på `factory/trondheim/+/+/telemetry` og skriver til TimescaleDB.

**Alarm engine** poller siste verdier og lager WARNING / CRITICAL ved terskelbrudd (temperatur, vibrasjon, trykk, RPM-avvik).

**Security monitor** fanger nettverkstrafikk eller kjører syntetisk demo. Ukjent IP som sender Modbus Write mot PLS-subnet flagges som HIGH og kan opprette en incident.

**Backend** er et REST-API mot dashboardet: sites, assets, telemetri, alarmer, security events, incidents, samt register/login med roller (admin / operator).

---

## Tech stack

| Lag | Teknologi |
| --- | --- |
| Backend | Rust, Axum, Tokio, SQLx |
| Frontend | React, TypeScript, Next.js, Tailwind |
| ML | Python, scikit-learn Isolation Forest |
| Data | PostgreSQL, TimescaleDB |
| Messaging | MQTT |
| OT | MQTT, Modbus TCP, OPC UA |
| Infra | Docker Compose, Prometheus, Grafana, GitHub Actions |

## Kom i gang

**Hele stakken i Docker (anbefalt for demo):**

```bash
docker compose up --build
```

- Dashboard: http://localhost:3001
- Backend: http://localhost:3000
- Grafana: http://localhost:3002 (admin / industrial)
- Prometheus: http://localhost:9090

På Windows uten containere for appene: `.\start-all.ps1` (krever Docker for MQTT + TimescaleDB, Rust, Node.js og Python 3.11+). Stopp med `.\stop-all.ps1`. Se `START.md`.

## Struktur

```text
apps/          frontend, backend
services/      ingestion, alerts, security-monitor
simulators/    factory-simulator
ml/            anomaly-detection + predictive maintenance
infrastructure/ docker, monitoring
```
