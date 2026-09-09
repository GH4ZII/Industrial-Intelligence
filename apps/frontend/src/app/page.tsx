import Link from "next/link";
import { getDashboard } from "@/lib/api";

function statusClass(status: string) {
  switch (status) {
    case "CRITICAL":
      return "text-red-400";
    case "WARNING":
      return "text-amber-400";
    default:
      return "text-emerald-400";
  }
}

export default async function Home() {
  const dashboard = await getDashboard();

  return (
    <main className="min-h-screen bg-neutral-950 text-white p-8">
      <div className="max-w-6xl mx-auto">
        <h1 className="text-3xl font-bold mb-8">
          Industrial Intelligence Platform
        </h1>

        <section className="mb-10 grid grid-cols-1 gap-8 sm:grid-cols-2">
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
        </section>

        <section>
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
                </div>

                <span className={`text-sm font-semibold ${statusClass(asset.status)}`}>
                  {asset.status}
                </span>
              </Link>
            ))}
          </div>
        </section>
      </div>
    </main>
  );
}
