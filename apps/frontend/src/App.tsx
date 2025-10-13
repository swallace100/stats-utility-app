import { useEffect, useState } from "react";

import NavBar from "./components/NavBar";
import ResultBoard from "./components/ResultBoard";
import UploadCsvCard from "./components/UploadCsvCard";
import {
  ping,
  readCsvFile,
  statsSummaryFromCsv,
  statsDistributionFromCsv,
} from "./lib/api";
import * as Api from "./lib/api";

// ----------------------------
// Main Application
// ----------------------------
export default function App() {
  const [healthy, setHealthy] = useState<boolean | null>(null);

  const [summary, setSummary] = useState<Api.SummaryOut | null>(null);
  const [dist, setDist] = useState<Api.DistOut | null>(null);
  const [ecdf, setEcdf] = useState<Api.EcdfOut | null>(null);
  const [qq, setQq] = useState<Api.QqOut | null>(null);

  const [busy, setBusy] = useState<boolean>(false);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    ping()
      .then((r) => setHealthy(r.ok))
      .catch(() => setHealthy(false));
  }, []);

  // ----------------------------
  // File analysis handler
  // ----------------------------
  async function onQuickAnalyze(file: File): Promise<void> {
    setBusy(true);
    setErr(null);

    try {
      const csv = await readCsvFile(file);

      const s = await statsSummaryFromCsv(csv);
      setSummary(s);

      const d = await statsDistributionFromCsv(csv);
      setDist(d);

      // TODO: add ECDF/QQ once Rust endpoints are ready
      setEcdf(null);
      setQq(null);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : "Analyze failed";
      setErr(msg);
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="min-h-screen flex flex-col">
      <NavBar />
      <main className="flex-1 flex flex-col items-center justify-start px-4 py-8">
        <div className="mx-auto w-full max-w-6xl space-y-8">
          {/* Header */}
          <section className="text-center">
            <h1 className="text-3xl font-semibold mb-2">Statistics Utility</h1>
            <p className="text-base text-neutral-700 mb-2">
              Upload a CSV with numbers to calculate statistics and render
              charts.
            </p>
            <p className="text-sm text-neutral-600">
              Backend health:{" "}
              <span
                className={
                  healthy
                    ? "text-green-600"
                    : healthy === false
                      ? "text-red-600"
                      : "text-neutral-500"
                }
              >
                {healthy === null ? "checking..." : healthy ? "OK" : "DOWN"}
              </span>
            </p>
          </section>

          {/* Upload + QuickAnalyze */}
          <section className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <div className="lg:col-span-2">
              <UploadCsvCard onAnalyzeFile={onQuickAnalyze} />
            </div>
            <div className="lg:col-span-1">
              <QuickAnalyze
                onAnalyze={onQuickAnalyze}
                busy={busy}
                error={err}
              />
            </div>
          </section>

          {/* Results */}
          <section>
            <ResultBoard summary={summary} dist={dist} ecdf={ecdf} qq={qq} />
          </section>
        </div>
      </main>

      <footer className="border-t bg-white">
        <div className="mx-auto max-w-6xl px-4 h-12 flex items-center text-xs text-neutral-500">
          © {new Date().getFullYear()} Stats Utility
        </div>
      </footer>
    </div>
  );
}

// ----------------------------
// QuickAnalyze component
// ----------------------------
function QuickAnalyze({
  onAnalyze,
  busy,
  error,
}: {
  // eslint-disable-next-line no-unused-vars
  onAnalyze: (_file: File) => void | Promise<void>;
  busy: boolean;
  error: string | null;
}) {
  const [file, setFile] = useState<File | null>(null);

  const handleClick = async (): Promise<void> => {
    if (!file) return;
    console.log(`Analyzing file: ${file.name}`); // ✅ actively uses file
    await onAnalyze(file);
  };

  return (
    <div className="rounded-2xl border border-neutral-200 p-4 bg-white shadow-sm">
      <h2 className="text-lg font-semibold mb-3">Quick Analyze</h2>
      <p className="text-sm text-neutral-600 mb-3">
        Analyze your CSV immediately via <code>stats_rs</code> and render plots
        via <code>plots_py</code>.
      </p>
      <input
        type="file"
        accept=".csv,text/csv"
        className="block w-full text-sm mb-3"
        onChange={(e) => setFile(e.target.files?.[0] ?? null)}
      />
      <button
        disabled={!file || busy}
        onClick={handleClick} // ✅ uses file properly now
        className={`w-full rounded-xl px-4 py-2 text-sm font-medium ${
          busy
            ? "bg-neutral-300 text-neutral-700"
            : "bg-black text-white hover:opacity-90"
        }`}
      >
        {busy ? "Analyzing…" : "Analyze"}
      </button>
      {error && <p className="text-sm text-red-600 mt-2">{error}</p>}
    </div>
  );
}
