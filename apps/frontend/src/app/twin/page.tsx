import TwinView from "@/components/TwinView";

export default function TwinPage() {
  return (
    <main className="min-h-screen p-8">
      <div className="max-w-6xl mx-auto">
        <p className="text-neutral-400 uppercase tracking-wide">Factory Trondheim</p>
        <h1 className="text-3xl font-bold mt-2 mb-2">Digital Twin</h1>
        <p className="text-neutral-400 mb-10">Production Line A</p>
        <TwinView />
      </div>
    </main>
  );
}
