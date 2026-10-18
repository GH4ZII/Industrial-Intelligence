import Link from "next/link";
import { getIncidents } from "@/lib/api";

function severityClass(severity: string) {
  switch (severity) {
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

export default async function IncidentsPage() {
  const incidents = await getIncidents();

  return (
    <main className="min-h-screen p-8">
      <div className="max-w-6xl mx-auto">
        <h1 className="text-3xl font-bold mb-8">Incidents</h1>

        {incidents.length === 0 ? (
          <p className="text-neutral-500">No incidents yet.</p>
        ) : (
          <div className="space-y-3">
            {incidents.map((incident) => (
              <Link
                key={incident.id}
                href={`/incidents/${incident.id}`}
                className="block border border-neutral-800 rounded-lg p-4 hover:border-neutral-600"
              >
                <div className="flex items-center justify-between gap-4">
                  <p className="font-semibold">
                    INCIDENT #{incident.id}
                  </p>
                  <span className={`text-sm font-semibold ${severityClass(incident.severity)}`}>
                    {incident.severity}
                  </span>
                </div>
                <p className="mt-2">{incident.title}</p>
                <p className="text-neutral-400 text-sm mt-2">
                  Asset: {incident.asset_id ?? "—"} · Status: {incident.status}
                  {incident.assigned_to ? ` · Assigned: ${incident.assigned_to}` : ""}
                  {incident.mitre_id
                    ? ` · ${incident.mitre_id} – ${incident.mitre_name}`
                    : ""}
                </p>
              </Link>
            ))}
          </div>
        )}
      </div>
    </main>
  );
}
