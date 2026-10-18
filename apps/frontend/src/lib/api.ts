const API_URL =
  process.env.NEXT_PUBLIC_API_URL?.replace(/\/$/, "") || "http://localhost:3000";

export interface Asset {
  site: string;
  asset_type: string;
  asset_id: string;
  status: string;
  health_percent?: number | null;
  anomaly_score?: number | null;
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

export interface AnomalyScore {
  time: string;
  site: string;
  asset_id: string;
  score: number;
  is_anomaly: boolean;
  contributing_metrics: string | null;
  message: string | null;
}

export interface AssetHealth {
  time: string;
  site: string;
  asset_id: string;
  health_percent: number;
  risk_7d: number;
  risk_30d: number;
  risk_90d: number;
  recommended_action: string | null;
}

export interface Incident {
  id: number;
  created_at: string;
  updated_at: string;
  severity: string;
  status: string;
  title: string;
  asset_id: string | null;
  description: string | null;
  assigned_to: string | null;
  mitre_id: string | null;
  mitre_name: string | null;
}

export interface IncidentEvent {
  id: number;
  incident_id: number;
  time: string;
  message: string;
}

export interface IncidentDetail {
  incident: Incident;
  events: IncidentEvent[];
}

export interface NetworkEvent {
  id: number;
  time: string;
  source_ip: string;
  destination_ip: string;
  source_port: number;
  destination_port: number;
  transport_protocol: string;
  application_protocol: string | null;
  payload_size: number;
  modbus_function: string | null;
  opcua_message: string | null;
  suspicious: boolean;
  severity: string | null;
  message: string | null;
}

async function getJson<T>(path: string, fallback?: T): Promise<T> {
  const response = await fetch(`${API_URL}${path}`, { cache: "no-store" });
  if (!response.ok) {
    if (fallback !== undefined) {
      return fallback;
    }
    throw new Error(`Failed to fetch ${path}`);
  }
  return response.json();
}

export async function getDashboard(): Promise<DashboardSummary> {
  return getJson("/api/dashboard");
}

export async function getAssets(): Promise<Asset[]> {
  return getJson("/api/assets");
}

export async function getAsset(assetId: string): Promise<AssetDetail> {
  return getJson(`/api/assets/${assetId}`);
}

export async function getAssetTelemetry(
  assetId: string
): Promise<TelemetryRow[]> {
  return getJson(`/api/assets/${assetId}/telemetry`);
}

export async function getAlerts(): Promise<Alert[]> {
  return getJson("/api/alerts");
}

export async function getAssetAnomalies(
  assetId: string
): Promise<AnomalyScore[]> {
  return getJson(`/api/assets/${assetId}/anomalies`, []);
}

export async function getAssetHealth(
  assetId: string
): Promise<AssetHealth | null> {
  const response = await fetch(`${API_URL}/api/assets/${assetId}/health`, {
    cache: "no-store",
  });
  if (!response.ok) {
    return null;
  }
  return response.json();
}

export async function getHealthOverview(): Promise<AssetHealth[]> {
  return getJson("/api/health-overview", []);
}

export async function getIncidents(): Promise<Incident[]> {
  return getJson("/api/incidents", []);
}

export async function getIncident(id: number | string): Promise<IncidentDetail> {
  return getJson(`/api/incidents/${id}`);
}

export async function patchIncident(
  id: number | string,
  body: { status?: string; assigned_to?: string }
): Promise<Incident> {
  const response = await fetch(`${API_URL}/api/incidents/${id}`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!response.ok) {
    throw new Error("Failed to update incident");
  }
  return response.json();
}

export async function getSecurityEvents(): Promise<NetworkEvent[]> {
  return getJson("/api/security/events", []);
}
