import Link from "next/link";
import { getDashboard, getIncidents, getSecurityEvents } from "@/lib/api";

function statusClass(status: string) {
  switch (status) {
    case "CRITICAL":
    case "HIGH":
      return "text-red-400";
    case "WARNING":
    case "MEDIUM":
      return "text-amber-400";
    default:
      return "text-emerald-400";
  }
}

function healthBar(percent: number) {
  const filled = Math.round(percent / 10);
  return "█".repeat(filled) + "░".repeat(Math.max(0, 10 - filled));
}

export default async function Home() {
  const [dashboard, securityEvents, incidents] = await Promise.all([
    getDashboard(),
    getSecurityEvents(),
    getIncidents(),
  ]);

  const suspicious = securityEvents.filter((e) => e.suspicious).slice(0, 5);
  const openIncidents = incidents
    .filter((incident) => incident.status !== "resolved")
    .slice(0, 5);

  return (
    <main className="min-h-screen p-8">
      <div className="max-w-6xl mx-auto">
        <h1 className="text-3xl font-bold mb-8">
          Industrial Intelligence Platform
        </h1>

        <section className="mb-10 grid grid-cols-1 gap-8 sm:grid-cols-3">
          <div>
            <p className="text-neutral-400">Plant Health</p>
            <p className="text-4xl font-bold mt-1">
              {dashboard.plant_health_percent}% Healthy
            </p>
          </div>

          <div>
            <p className="text-neutral-400">Security</p>
            <p className="text-4xl font-bold mt-1">
              {dashboard.active_alerts} Active Alerts
            </p>
          </div>

          <div>
            <p className="text-neutral-400">Incidents</p>
            <p className="text-4xl font-bold mt-1">
              {openIncidents.length} Open
            </p>
          </div>
        </section>

        <section className="mb-10">
          <h2 className="text-xl font-semibold mb-4">Machines</h2>

          <div className="space-y-3">
            {dashboard.assets.map((asset) => (
              <Link
                key={asset.asset_id}
                href={`/assets/${asset.asset_id}`}
                className="flex items-center justify-between border border-neutral-800 rounded-lg p-4 hover:border-neutral-600"
              >
                <div>
                  <p className="font-semibold">
                    {asset.asset_id}{" "}
                    <span className="text-neutral-400 font-normal capitalize">
                      {asset.asset_type}
                    </span>
                  </p>
                  <p className="text-neutral-500 text-sm">{asset.site}</p>
                  {asset.health_percent != null && (
                    <p className="text-neutral-400 text-sm mt-1 font-mono">
                      Health {healthBar(asset.health_percent)}{" "}
                      {Math.round(asset.health_percent)}%
                    </p>
                  )}
                </div>

                <span
                  className={`text-sm font-semibold ${statusClass(asset.status)}`}
                >
                  {asset.status}
                </span>
              </Link>
            ))}
          </div>
        </section>

        <section className="mb-10">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-xl font-semibold">Open Incidents</h2>
            <Link href="/incidents" className="text-sm text-neutral-400 hover:text-white">
              View all
            </Link>
          </div>

          {openIncidents.length === 0 ? (
            <p className="text-neutral-500">No open incidents.</p>
          ) : (
            <div className="space-y-3">
              {openIncidents.map((incident) => (
                <Link
                  key={incident.id}
                  href={`/incidents/${incident.id}`}
                  className="block border border-neutral-800 rounded-lg p-4 hover:border-neutral-600"
                >
                  <div className="flex items-center justify-between gap-4">
                    <p className="font-semibold">
                      INCIDENT #{incident.id} · {incident.title}
                    </p>
                    <span className={`text-sm font-semibold ${statusClass(incident.severity)}`}>
                      {incident.severity}
                    </span>
                  </div>
                  <p className="text-neutral-400 text-sm mt-2">
                    {incident.asset_id ?? "Unknown asset"} · {incident.status}
                    {incident.assigned_to ? ` · ${incident.assigned_to}` : ""}
                  </p>
                </Link>
              ))}
            </div>
          )}
        </section>

        <section>
          <h2 className="text-xl font-semibold mb-4">OT Security Events</h2>

          {suspicious.length === 0 ? (
            <p className="text-neutral-500">
              No suspicious Modbus/OPC UA events yet.
            </p>
          ) : (
            <div className="space-y-3">
              {suspicious.map((event) => (
                <div
                  key={event.id}
                  className="border border-neutral-800 rounded-lg p-4"
                >
                  <div className="flex items-center justify-between gap-4">
                    <p className="font-semibold">
                      {event.message ?? "Security event"}
                    </p>
                    <span
                      className={`text-sm font-semibold ${statusClass(
                        event.severity ?? "HIGH"
                      )}`}
                    >
                      {event.severity ?? "HIGH"}
                    </span>
                  </div>
                  <p className="text-neutral-400 text-sm mt-2">
                    {event.source_ip} → {event.destination_ip}:
                    {event.destination_port}
                    {event.modbus_function
                      ? ` · ${event.modbus_function}`
                      : ""}
                    {event.opcua_message
                      ? ` · OPC UA ${event.opcua_message}`
                      : ""}
                  </p>
                </div>
              ))}
            </div>
          )}
        </section>
      </div>
    </main>
  );
}
