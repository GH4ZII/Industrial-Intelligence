"use client";

import { useEffect, useState } from "react";

import {
  AnomalyScore,
  AssetDetail,
  AssetHealth,
  TelemetryRow,
  getAsset,
  getAssetAnomalies,
  getAssetHealth,
  getAssetTelemetry,
} from "@/lib/api";

import TelemetryChart from "@/components/TelemetryChart";

interface Props {
  assetId: string;
}

function healthBar(percent: number) {
  const filled = Math.round(percent / 10);
  return "█".repeat(filled) + "░".repeat(Math.max(0, 10 - filled));
}

export default function LiveTelemetry({ assetId }: Props) {
  const [asset, setAsset] = useState<AssetDetail | null>(null);
  const [telemetry, setTelemetry] = useState<TelemetryRow[]>([]);
  const [anomalies, setAnomalies] = useState<AnomalyScore[]>([]);
  const [health, setHealth] = useState<AssetHealth | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function load() {
      try {
        const [assetData, telemetryData, anomalyData, healthData] =
          await Promise.all([
            getAsset(assetId),
            getAssetTelemetry(assetId),
            getAssetAnomalies(assetId),
            getAssetHealth(assetId),
          ]);

        if (cancelled) {
          return;
        }

        setAsset(assetData);
        setTelemetry(telemetryData);
        setAnomalies(anomalyData);
        setHealth(healthData);
        setError(null);
      } catch (loadError) {
        console.error(loadError);
        if (!cancelled) {
          setError("Failed to load telemetry");
        }
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
  }, [assetId]);

  if (error) {
    return <p>{error}</p>;
  }

  if (!asset) {
    return <p>Loading telemetry...</p>;
  }

  const metricData = (metric: string) =>
    telemetry
      .filter((row) => row.metric === metric)
      .reverse()
      .map((row) => ({
        time: new Date(row.time).toLocaleTimeString(),
        value: row.value,
      }));

  const anomalySeries = [...anomalies]
    .reverse()
    .map((row) => ({
      time: new Date(row.time).toLocaleTimeString(),
      value: row.score,
    }));

  const latestScore = anomalies[0]?.score;

  return (
    <>
      {health && (
        <section className="mb-10">
          <h2 className="text-xl font-semibold mb-4">
            Predictive Maintenance
          </h2>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="border border-neutral-800 rounded-lg p-5">
              <p className="text-neutral-400">Health</p>
              <p className="text-3xl font-semibold mt-2">
                {Math.round(health.health_percent)}%
              </p>
              <p className="font-mono text-neutral-300 mt-2">
                {healthBar(health.health_percent)}
              </p>
              {latestScore != null && (
                <p className="text-neutral-400 text-sm mt-3">
                  Anomaly score {latestScore.toFixed(2)}
                  {latestScore >= 0.7 ? " · Potential equipment failure" : ""}
                </p>
              )}
            </div>

            <div className="border border-neutral-800 rounded-lg p-5">
              <p className="text-neutral-400">Estimated failure risk</p>
              <ul className="mt-3 space-y-1 text-sm">
                <li>7 days: {health.risk_7d.toFixed(0)}%</li>
                <li>30 days: {health.risk_30d.toFixed(0)}%</li>
                <li>90 days: {health.risk_90d.toFixed(0)}%</li>
              </ul>
              {health.recommended_action && (
                <p className="text-neutral-300 mt-4">
                  Recommended action
                  <br />
                  <span className="text-white">
                    {health.recommended_action}
                  </span>
                </p>
              )}
            </div>
          </div>
        </section>
      )}

      <section>
        <h2 className="text-xl font-semibold mb-4">
          Current Telemetry
        </h2>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {Object.entries(asset.latest).map(([metric, value]) => (
            <div
              key={metric}
              className="border border-neutral-800 rounded-lg p-5"
            >
              <p className="text-neutral-400 capitalize">
                {metric}
              </p>

              <p className="text-2xl font-semibold mt-2">
                {value.toFixed(2)}
              </p>
            </div>
          ))}
        </div>
      </section>

      <section className="mt-10">
        <h2 className="text-xl font-semibold mb-4">
          Live Telemetry
        </h2>

        <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
          {anomalySeries.length > 0 && (
            <TelemetryChart
              title="Anomaly score"
              data={anomalySeries}
            />
          )}

          {asset.latest.temperature !== undefined && (
            <TelemetryChart
              title="Temperature"
              unit="°C"
              data={metricData("temperature")}
            />
          )}

          {asset.latest.vibration !== undefined && (
            <TelemetryChart
              title="Vibration"
              unit="mm/s"
              data={metricData("vibration")}
            />
          )}

          {asset.latest.rpm !== undefined && (
            <TelemetryChart
              title="RPM"
              unit="RPM"
              data={metricData("rpm")}
            />
          )}

          {asset.latest.pressure !== undefined && (
            <TelemetryChart
              title="Pressure"
              unit="bar"
              data={metricData("pressure")}
            />
          )}

          {asset.latest.power !== undefined && (
            <TelemetryChart
              title="Power"
              unit="kW"
              data={metricData("power")}
            />
          )}

          {asset.latest.current !== undefined && (
            <TelemetryChart
              title="Current"
              unit="A"
              data={metricData("current")}
            />
          )}

          {asset.latest.level !== undefined && (
            <TelemetryChart
              title="Level"
              unit="%"
              data={metricData("level")}
            />
          )}

          {asset.latest.flow !== undefined && (
            <TelemetryChart
              title="Flow"
              data={metricData("flow")}
            />
          )}

          {asset.latest.position !== undefined && (
            <TelemetryChart
              title="Position"
              unit="%"
              data={metricData("position")}
            />
          )}
        </div>
      </section>
    </>
  );
}
