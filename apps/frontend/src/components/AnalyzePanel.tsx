import { useState } from "react";

export function AnalyzePanel({
  busy,
  error,
  onCalculate,
}: {
  busy: boolean;
  error: string | null;
  // eslint-disable-next-line no-unused-vars
  onCalculate: (file: File) => void | Promise<void>;
}) {
  const [file, setFile] = useState<File | null>(null);

  async function handleClick(): Promise<void> {
    if (!file) return;
    await onCalculate(file);
  }

  return (
    <section className="rounded-2xl border border-neutral-200 bg-white/90 backdrop-blur-md shadow-[0_20px_50px_-12px_rgba(0,0,0,0.12)] ring-1 ring-black/[0.03] p-8 max-w-2xl w-full mx-auto">
      {/* Header */}
      <header className="text-center mb-8 space-y-3">
        <h1 className="text-3xl font-semibold text-neutral-900 tracking-tight">
          Stats Utility
        </h1>

        <p className="text-base text-neutral-700 leading-relaxed max-w-lg mx-auto">
          Upload a CSV file to calculate summary statistics and generate plots.
        </p>

        <p className="text-base text-neutral-700 leading-relaxed max-w-lg mx-auto">
          All results will appear below.
        </p>

        <hr />
        {/* Inline tips */}
        <ul className="text-sm text-neutral-600 leading-relaxed space-y-1 max-w-md mx-auto">
          <li>• Numeric columns only.</li>
          <li>• Keep files under ~5 MB for best performance.</li>
        </ul>
      </header>

      {/* File input */}
      <div className="mb-6">
        <label className="text-sm font-medium text-neutral-800 block mb-2">
          Select CSV file
        </label>
        <input
          type="file"
          accept=".csv,text/csv"
          className="block w-full rounded-lg border border-[#60a5fa] bg-[#f0f8ff] px-3 py-2 text-sm text-neutral-900 shadow-sm focus:outline-none focus:ring-2 focus:ring-[#3b82f6] focus:border-[#2563eb] transition"
          onChange={(e) => setFile(e.target.files?.[0] ?? null)}
        />
      </div>

      {/* Calculate button */}
      <button
        disabled={!file || busy}
        onClick={handleClick}
        className={`w-full rounded-xl px-4 py-2.5 text-sm font-medium shadow-sm transition ${
          busy || !file
            ? "bg-neutral-200 text-neutral-500 cursor-not-allowed"
            : "bg-[#2563eb] hover:bg-[#1d4ed8] text-white"
        }`}
      >
        {busy ? "Calculating…" : "Calculate"}
      </button>

      {/* Error message */}
      {error && (
        <p className="text-sm text-red-600 mt-4 leading-snug text-center">
          {error}
        </p>
      )}
    </section>
  );
}
