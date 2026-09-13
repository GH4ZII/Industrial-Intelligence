# Industrial Intelligence

Plattform for overvåking av industrielle systemer og OT-miljøer. Samler sanntidsdata fra maskiner og sensorer, oppdager unormal oppførsel, og gir operatører oversikt over både produksjon og cybersikkerhet.

Målet er programvare som kunne brukes i norsk industri, energi, olje/gass, maritim sektor eller kritisk infrastruktur.

## Hva systemet gjør

- Innhenter telemetri via MQTT, Modbus TCP og OPC UA
- Prosesserer sanntids- og historiske sensordata
- Viser plant health, digital twin og asset-status
- Genererer alarmer ved terskelbrudd og ML-anomalier
- Estimerer maskinhelse og vedlikeholdsrisiko
- Overvåker OT-nettverk for mistenkelig aktivitet (MITRE ATT&CK for ICS)
- Støtter incident management og predictive maintenance

## Arkitektur

```text
PLS / simulatorer / sensorer
        │
        ├── Modbus TCP
        ├── OPC UA
        └── MQTT
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
