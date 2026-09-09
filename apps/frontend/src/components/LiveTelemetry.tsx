"use client";

import { useEffect, useState } from "react";

import {
  AssetDetail,
  TelemetryRow,
  getAsset,
  getAssetTelemetry,
} from "@/lib/api";

import TelemetryChart from "@/components/TelemetryChart";

interface Props {
  assetId: string;
}

export default function LiveTelemetry({ assetId }: Props) {
  const [asset, setAsset] = useState<AssetDetail | null>(null);
  const [telemetry, setTelemetry] = useState<TelemetryRow[]>([]);
  const [error, setError] = useState<string | null>(null);

  async function loadData() {
    try {
      const [assetData, telemetryData] = await Promise.all([
        getAsset(assetId),
        getAssetTelemetry(assetId),
      ]);

      setAsset(assetData);
      setTelemetry(telemetryData);
      setError(null);
    } catch (error) {
      console.error(error);
      setError("Failed to load telemetry");
    }
  }

  useEffect(() => {
    loadData();

    const interval = setInterval(() => {
      loadData();
    }, 2000);

    return () => clearInterval(interval);
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

  return (
    <>
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