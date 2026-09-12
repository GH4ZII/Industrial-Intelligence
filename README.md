# Industrial Intelligence

Plattform for overvåking av industrielle systemer og OT-miljøer. Samler sanntidsdata fra maskiner og sensorer, oppdager unormal oppførsel, og gir operatører oversikt over både produksjon og cybersikkerhet.

Målet er programvare som kunne brukes i norsk industri, energi, olje/gass, maritim sektor eller kritisk infrastruktur.

## Hva systemet gjør

- Innhenter telemetri via MQTT, Modbus TCP og OPC UA
- Prosesserer sanntids- og historiske sensordata
- Viser plant health og asset-status i et dashboard
- Genererer alarmer ved terskelbrudd og anomalier
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
 Industrial Gateway → Message Broker (MQTT / Kafka)
        │
        ├─ Data Processing → Time-Series DB
        └─ Security Monitor → Alert Engine
                    │
              Backend API → Dashboard
```

## Tech stack

| Lag | Teknologi |
| --- | --- |
| Backend | Rust, Axum, Tokio, SQLx |
| Frontend | React, TypeScript, Next.js, Tailwind |
| Data | PostgreSQL, TimescaleDB |
| Messaging | MQTT (Kafka senere) |
| OT | MQTT, Modbus TCP, OPC UA |
| Infra | Docker, Compose, Prometheus, Grafana |

## MVP

Første fungerende versjon:

```text
Industrial Simulator → MQTT → Rust backend → TimescaleDB → React dashboard
```

Dashboardet viser maskiner, temperatur, vibrasjon, RPM, trykk, status og historiske grafer. OT Security bygges etter at MVP er stabil.

## Milepæler

| Uke | Milepæl |
| --- | --- |
| 1–2 | Industrial simulator |
| 3 | MQTT pipeline |
| 4–5 | Rust backend |
| 6–7 | Dashboard |
| 8 | Alarm engine |
| 9–10 | OT network monitor |
| 11 | Cyber detection |
| 12–13 | AI anomaly detection |
| 14 | Digital twin |
| 15 | Incident management |
| 16 | Full deployment |

## Målstruktur

```text
apps/          frontend, backend
services/      ingestion, alerts, security, anomaly, incidents
simulators/    factory- og PLC-simulator
protocols/     mqtt, modbus, opcua
infrastructure/ docker, k8s, monitoring
ml/            anomaly detection, predictive maintenance
docs/          architecture, security, api
```

## Status

Tidlig fase. Kodebase og arkitektur er under oppsett.

## Kom i gang

```bash
git clone https://github.com/<org>/Industrial-Intelligence.git
cd Industrial-Intelligence
```

På Windows (PowerShell), start hele stakken:

```powershell
.\start-all.ps1
```

Dashboard: http://localhost:3001 — stopp med `.\stop-all.ps1`. Se `START.md` for detaljer.

## Merknad

Detaljert prosjektplan ligger lokalt i `project-plan.md` og er ikke versjonert.
