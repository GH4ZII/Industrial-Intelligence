"use client";

import Link from "next/link";
import { useEffect, useState } from "react";

import { Asset, getAssets } from "@/lib/api";

const LINE = [
  { id: "P101", subtitle: "Pump 01" },
  { id: "T301", subtitle: "Tank 01" },
  { id: "M201", subtitle: "Motor 1" },
];

const VALVE = { id: "V401", type: "Valve", subtitle: "Valve 01" };

function statusColor(status?: string) {
  if (status === "CRITICAL" || status === "HIGH") {
    return "bg-red-500 border-red-400";
  }
  if (status === "WARNING" || status === "MEDIUM") {
    return "bg-amber-400 border-amber-300";
  }
  return "bg-emerald-500 border-emerald-400";
}

function NodeCard({
  asset,
  label,
  subtitle,
}: {
  asset?: Asset;
  label: string;
  subtitle: string;
}) {
  const color = statusColor(asset?.status);
  const href = `/assets/${label}`;

  return (
    <Link
      href={href}
      className="block w-44 border border-neutral-700 rounded-lg bg-neutral-900 p-4 hover:border-neutral-500"
    >
      <div className={`h-3 w-3 rounded-full mb-3 ${color}`} />
      <p className="text-xs uppercase text-neutral-500">{asset?.asset_type ?? subtitle}</p>
      <p className="text-xl font-semibold">{label}</p>
      <p className="text-sm text-neutral-400 mt-1">{asset?.status ?? "UNKNOWN"}</p>
      {asset?.health_percent != null && (
        <p className="text-xs text-neutral-500 mt-2">
          Health {Math.round(asset.health_percent)}%
        </p>
      )}
    </Link>
  );
}

export default function TwinView() {
  const [assets, setAssets] = useState<Asset[]>([]);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      try {
        const next = await getAssets();
        if (!cancelled) {
          setAssets(next);
        }
      } catch (error) {
        console.error(error);
      }
    }

    const interval = setInterval(() => {
      void load();
    }, 2000);
    const initial = setTimeout(() => {
      void load();
    }, 0);

    return () => {
      cancelled = true;
      clearInterval(interval);
      clearTimeout(initial);
    };
  }, []);

  const byId = Object.fromEntries(assets.map((asset) => [asset.asset_id, asset]));

  return (
    <div>
      <p className="text-neutral-400 mb-8">
        GREEN = normal · YELLOW = warning · RED = critical
      </p>

      <div className="flex flex-col items-start gap-4">
        {LINE.map((node, index) => (
          <div key={node.id} className="flex flex-col items-center">
            <NodeCard
              asset={byId[node.id]}
              label={node.id}
              subtitle={node.subtitle}
            />
            {index < LINE.length - 1 && (
              <div className="h-10 w-px bg-neutral-600 my-1" />
            )}
          </div>
        ))}
      </div>

      <div className="mt-10">
        <p className="text-neutral-500 text-sm mb-3">Auxiliary</p>
        <NodeCard
          asset={byId[VALVE.id]}
          label={VALVE.id}
          subtitle={VALVE.subtitle}
        />
      </div>
    </div>
  );
}
