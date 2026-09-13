# Industrial Intelligence

Overvåkingsplattform for industri og OT-miljøer. Den samler sanntidsdata fra maskiner, viser anleggshelse i et dashboard, lager alarmer ved unormal drift, og overvåker samtidig OT-nettverket for mistenkelig aktivitet mot PLS-er.

Målet er programvare som kunne brukes i norsk industri, energi, olje/gass, maritim sektor eller kritisk infrastruktur — der produksjon og cybersikkerhet må ses sammen.

---

## Problemet

I en fabrikk kjører pumper, motorer, tanker og ventiler. Sensorer sender temperatur, vibrasjon, trykk og RPM. Hvis en pumpe begynner å vibrere for mye, må operatøren se det raskt.

Samtidig er OT-nettverket (PLS, HMI, Modbus, OPC UA) et eget angrepsflate. En ukjent maskin som skriver til en PLS kan stoppe eller sabotere produksjonen. Vanlige IT-overvåkingssystemer ser sjelden denne trafikken.

Industrial Intelligence kombinerer **plant health** og **OT-sikkerhet** i ett bilde.

---

## Hva som er bygget

En virtuell fabrikk i Trondheim simulerer fire maskiner og publiserer telemetri hvert sekund. Dataene går gjennom en MQTT-pipeline, lagres som tidsserier, evalueres mot terskelregler, og vises i et dashboard. En egen sikkerhetstjeneste parser Modbus TCP og OPC UA og flagger uautoriserte skriv mot PLS.

| Asset | Type | Målinger |
| --- | --- | --- |
| P101 | Pumpe | RPM, temperatur, vibrasjon, trykk, effekt |
| M201 | Motor | temperatur, RPM, strøm, vibrasjon |
| T301 | Tank | nivå, temperatur, trykk |
| V401 | Ventil | åpen/lukket, flow, posisjon |

Maskinene kan gå i feiltilstand: temperatur og vibrasjon stiger, RPM faller. Det gir realistiske alarmer uten ekte hardware.

Dashboardet viser plant health, maskinstatus (NORMAL / WARNING / CRITICAL), live grafer per asset, og OT-sikkerhetshendelser.

---

## Arkitektur

Tjenestene er løst koblet. Ingest, alarmer og pakkeanalyse er egne prosesser som kommuniserer via MQTT og databasen.

```text
Virtuell fabrikk (simulator)
        │  MQTT hvert sekund
        ▼
   Mosquitto (broker)
        │
        ▼
   Ingestion-service
        │
        ▼
   TimescaleDB  ──►  Alarm Engine (terskler)
        │
        ▼
   Backend API  ──►  Next.js-dashboard

Parallelt:
   Security Monitor (pcap / demo)
        │  parser Modbus TCP + OPC UA
        ▼
   network_events + incidents  ──►  dashboard
```

**Simulator** publiserer JSON på topics som `factory/trondheim/pump/P101/telemetry`.

**Ingestion** abonnerer på `factory/trondheim/+/+/telemetry` og skriver til TimescaleDB.

**Alarm engine** poller siste verdier og lager WARNING / CRITICAL ved terskelbrudd (temperatur, vibrasjon, trykk, RPM-avvik).

**Security monitor** fanger nettverkstrafikk eller kjører syntetisk demo. Ukjent IP som sender Modbus Write mot PLS-subnet flagges som HIGH og kan opprette en incident.

**Backend** er et REST-API mot dashboardet: sites, assets, telemetri, alarmer, security events, incidents, samt register/login med roller (admin / operator).

---

## Tech stack

| Lag | Teknologi | Hvorfor |
| --- | --- | --- |
| Tjenester | Rust, Tokio, Axum, SQLx | Lav latency og minnesikkerhet, passer industri og sikkerhetstjenester |
| Frontend | Next.js, React, TypeScript, Tailwind | Operatørdashboard med live grafer |
| Data | TimescaleDB (PostgreSQL) | Tidsserier uten egen Influx-stack |
| Meldinger | MQTT (Mosquitto) | Lett og vanlig i IoT/OT; Kafka er neste steg ved høyere volum |
| OT | Modbus TCP, OPC UA | Protokoller mot PLS og industrielle kontrollere |
| Infra | Docker Compose | MQTT-broker og database lokalt |

Alarmer er terskelbaserte først. Det er forklarbart og testbart. Maskinlæring for anomalier ligger i veikartet, ikke i koden ennå.

---

## Demo-scenario

1. Fabrikken kjører normalt. Dashboardet viser høy plant health.
2. Pumpe P101 går i fault. Vibrasjon og temperatur stiger.
3. Alarm engine lager CRITICAL. Plant health faller. Operatøren ser det på grafer.
4. Samtidig sender en ukjent IP en Modbus Write til en PLS.
5. Security monitor flagger HIGH, lager incident, og dashboardet viser OT-hendelsen.

Det er produksjon og cyber i samme system — det som skiller prosjektet fra et vanlig IoT-dashboard.

---

## Repo-struktur

```text
apps/
  backend/              REST-API (Axum)
  frontend/             Operatørdashboard (Next.js)
services/
  ingestion/            MQTT → TimescaleDB
  alerts/               Terskelalarmer
  security-monitor/     OT-trafikk, Modbus/OPC UA, deteksjon
simulators/
  factory-simulator/    Virtuell fabrikk
infrastructure/
  docker/               TimescaleDB-schema
```

---

## Veikart

Bygget: simulator, MQTT-pipeline, ingest, backend, dashboard, alarm engine, OT-monitor med deteksjon og incidents.

Ikke bygget ennå: Kafka, ML-anomalideteksjon, predictive maintenance, digital twin, MITRE ATT&CK for ICS-mapping, Kubernetes, Prometheus/Grafana, CI/CD. Telemetri går via MQTT; Modbus og OPC UA brukes i security-monitor.

---

## Kom i gang

```powershell
.\start-all.ps1
```

Dashboard: http://localhost:3001

Stopp med `.\stop-all.ps1`. Detaljert oppstart, porter og manuell kjøring står i [`START.md`](START.md).
