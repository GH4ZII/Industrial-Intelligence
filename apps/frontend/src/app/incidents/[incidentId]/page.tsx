import { notFound } from "next/navigation";
import IncidentActions from "@/components/IncidentActions";
import { getIncident } from "@/lib/api";

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

interface Props {
  params: Promise<{ incidentId: string }>;
}

export default async function IncidentDetailPage({ params }: Props) {
  const { incidentId } = await params;
  let detail;
  try {
    detail = await getIncident(incidentId);
  } catch {
    notFound();
  }

  const { incident, events } = detail;

  return (
    <main className="min-h-screen p-8">
      <div className="max-w-6xl mx-auto">
        <p className="text-neutral-400 uppercase tracking-wide">SOC</p>
        <h1 className="text-3xl font-bold mt-2">INCIDENT #{incident.id}</h1>

        <div className="mt-8 grid grid-cols-1 md:grid-cols-2 gap-6">
          <div className="border border-neutral-800 rounded-lg p-5 space-y-3">
            <p>
              <span className="text-neutral-400">Severity:</span>{" "}
              <span className={severityClass(incident.severity)}>
                {incident.severity}
              </span>
            </p>
            <p>
              <span className="text-neutral-400">Asset:</span>{" "}
              {incident.asset_id ?? "—"}
            </p>
            <p>
              <span className="text-neutral-400">Event:</span> {incident.title}
            </p>
            <p>
              <span className="text-neutral-400">Status:</span> {incident.status}
            </p>
            <p>
              <span className="text-neutral-400">Assigned:</span>{" "}
              {incident.assigned_to ?? "Unassigned"}
            </p>
          </div>

          <div className="border border-neutral-800 rounded-lg p-5">
            <p className="text-neutral-400">Attack Technique</p>
            {incident.mitre_id ? (
              <p className="text-xl font-semibold mt-2">
                {incident.mitre_id} – {incident.mitre_name}
              </p>
            ) : (
              <p className="text-neutral-500 mt-2">Not mapped</p>
            )}
            {incident.description && (
              <pre className="text-sm text-neutral-400 mt-4 whitespace-pre-wrap">
                {incident.description}
              </pre>
            )}
          </div>
        </div>

        <section className="mt-10">
          <h2 className="text-xl font-semibold mb-4">Incident timeline</h2>
          {events.length === 0 ? (
            <p className="text-neutral-500">No timeline events.</p>
          ) : (
            <div className="space-y-3">
              {events.map((event) => (
                <div
                  key={event.id}
                  className="border border-neutral-800 rounded-lg p-4 flex gap-4"
                >
                  <p className="text-neutral-500 font-mono text-sm w-24 shrink-0">
                    {new Date(event.time).toLocaleTimeString()}
                  </p>
                  <p>{event.message}</p>
                </div>
              ))}
            </div>
          )}
        </section>

        <IncidentActions incident={incident} />
      </div>
    </main>
  );
}
