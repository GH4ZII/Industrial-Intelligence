const API_URL = "http://localhost:3000";

export interface Asset {
  site: string;
  asset_type: string;
  asset_id: string;
  status: string;
}

export interface DashboardSummary {
  plant_health_percent: number;
  active_alerts: number;
  assets: Asset[];
}

export interface AssetDetail {
  site: string;
  asset_type: string;
  asset_id: string;
  latest: Record<string, number>;
}

export interface TelemetryRow {
  time: string;
  site: string;
  asset_type: string;
  asset_id: string;
  metric: string;
  value: number;
}

export interface Alert {
  id: number;
  time: string;
  site: string;
  asset_id: string;
  severity: string;
  alert_type: string;
  message: string;
  acknowledged: boolean;
}

export async function getDashboard(): Promise<DashboardSummary> {
  const response = await fetch(`${API_URL}/api/dashboard`, {
    cache: "no-store",
  });

  if (!response.ok) {
    throw new Error("Failed to fetch dashboard");
  }

  return response.json();
}

export async function getAssets(): Promise<Asset[]> {
  const response = await fetch(`${API_URL}/api/assets`, {
    cache: "no-store",
  });

  if (!response.ok) {
    throw new Error("Failed to fetch assets");
  }

  return response.json();
}

export async function getAsset(assetId: string): Promise<AssetDetail> {
  const response = await fetch(`${API_URL}/api/assets/${assetId}`, {
    cache: "no-store",
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch asset ${assetId}`);
  }

  return response.json();
}

export async function getAssetTelemetry(
  assetId: string
): Promise<TelemetryRow[]> {
  const response = await fetch(
    `${API_URL}/api/assets/${assetId}/telemetry`,
    {
      cache: "no-store",
    }
  );

  if (!response.ok) {
    throw new Error(`Failed to fetch telemetry for ${assetId}`);
  }

  return response.json();
}

export async function getAlerts(): Promise<Alert[]> {
  const response = await fetch(`${API_URL}/api/alerts`, {
    cache: "no-store",
  });

  if (!response.ok) {
    throw new Error("Failed to fetch alerts");
  }

  return response.json();
}
