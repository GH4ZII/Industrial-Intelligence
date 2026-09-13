"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { Incident, patchIncident } from "@/lib/api";

export default function IncidentActions({ incident }: { incident: Incident }) {
  const router = useRouter();
  const [status, setStatus] = useState(incident.status);
  const [assigned, setAssigned] = useState(incident.assigned_to ?? "");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function save() {
    setSaving(true);
    setError(null);
    try {
      await patchIncident(incident.id, {
        status,
        assigned_to: assigned || undefined,
      });
      router.refresh();
    } catch (saveError) {
      console.error(saveError);
      setError("Could not update incident");
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="border border-neutral-800 rounded-lg p-5 mt-8">
      <h2 className="font-semibold mb-4">Response</h2>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <label className="text-sm text-neutral-400">
          Status
          <select
            className="mt-1 w-full bg-neutral-900 border border-neutral-700 rounded px-3 py-2 text-white"
            value={status}
            onChange={(event) => setStatus(event.target.value)}
          >
            <option value="open">open</option>
            <option value="investigating">investigating</option>
            <option value="resolved">resolved</option>
          </select>
        </label>
        <label className="text-sm text-neutral-400">
          Assigned
          <input
            className="mt-1 w-full bg-neutral-900 border border-neutral-700 rounded px-3 py-2 text-white"
            value={assigned}
            onChange={(event) => setAssigned(event.target.value)}
            placeholder="OT Security Team"
          />
        </label>
      </div>
      <button
        type="button"
        onClick={save}
        disabled={saving}
        className="mt-4 px-4 py-2 rounded bg-white text-black text-sm font-semibold disabled:opacity-50"
      >
        {saving ? "Saving..." : "Update incident"}
      </button>
      {error && <p className="text-red-400 text-sm mt-3">{error}</p>}
    </div>
  );
}
