"use client";

import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
} from "recharts";

interface ChartPoint {
  time: string;
  value: number;
}

interface TelemetryChartProps {
  title: string;
  unit?: string;
  data: ChartPoint[];
}

export default function TelemetryChart({
  title,
  unit,
  data,
}: TelemetryChartProps) {
  return (
    <div className="border border-neutral-800 rounded-lg p-5">
      <h3 className="font-semibold mb-4">
        {title}
      </h3>

      <div className="h-64">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={data}>
            <XAxis
              dataKey="time"
              tick={{ fontSize: 12 }}
            />

            <YAxis
              tick={{ fontSize: 12 }}
            />

            <Tooltip
              formatter={(value) => [
                `${Number(value).toFixed(2)} ${unit ?? ""}`,
                title,
              ]}
            />

            <Line
              type="monotone"
              dataKey="value"
              dot={false}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}