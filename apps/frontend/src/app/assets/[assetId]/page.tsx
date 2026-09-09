import Link from "next/link";
import { getAsset } from "@/lib/api";
import LiveTelemetry from "@/components/LiveTelemetry";

interface AssetPageProps {
  params: Promise<{
    assetId: string;
  }>;
}

export default async function AssetPage({
  params,
}: AssetPageProps) {
  const { assetId } = await params;

  const asset = await getAsset(assetId);

  return (
    <main className="min-h-screen bg-neutral-950 text-white p-8">
      <div className="max-w-6xl mx-auto">

        <Link
          href="/"
          className="text-neutral-400 hover:text-white"
        >
          ← Back to dashboard
        </Link>

        <div className="mt-8 mb-10">
          <p className="text-neutral-400 uppercase">
            {asset.asset_type}
          </p>

          <h1 className="text-4xl font-bold">
            {asset.asset_id}
          </h1>

          <p className="text-neutral-400 mt-2">
            Site: {asset.site}
          </p>
        </div>

        <LiveTelemetry assetId={assetId} />

      </div>
    </main>
  );
}