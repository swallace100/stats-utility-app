import { useEffect, useState } from "react";

import { AnalyzePanel } from "./components/AnalyzePanel";
import NavBar from "./components/NavBar";
import ResultBoard from "./components/ResultBoard";
import {
  ping,
  readCsvFile,
  statsSummaryFromCsv,
  statsDistributionFromCsv,
} from "./lib/api";
import * as Api from "./lib/api";

// tiny health pill
function HealthBadge({ healthy }: { healthy: boolean | null }) {
  let wrap =
    "bg-neutral-300 text-neutral-700 ring-1 ring-inset ring-neutral-400";
  let dot = "bg-neutral-500";
  let label = "checking…";

  if (healthy === true) {
    wrap = "bg-emerald-100 text-emerald-700 ring-1 ring-inset ring-emerald-300";
    dot = "bg-emerald-500";
    label = "Online";
  } else if (healthy === false) {
    wrap = "bg-red-100 text-red-700 ring-1 ring-inset ring-red-300";
    dot = "bg-red-500";
    label = "Offline";
  }

  return (
    <span
      className={`inline-flex items-center gap-1 rounded-full px-2 py-1 text-[11px] font-medium leading-none ${wrap}`}
    >
      <span className={`h-1.5 w-1.5 rounded-full ${dot}`} />
      {label}
    </span>
  );
}

export default function App() {
  // health
  const [healthy, setHealthy] = useState<boolean | null>(null);

  // results
  const [summary, setSummary] = useState<Api.SummaryOut | null>(null);
  const [dist, setDist] = useState<Api.DistOut | null>(null);
  const [ecdf, setEcdf] = useState<Api.EcdfOut | null>(null);
  const [qq, setQq] = useState<Api.QqOut | null>(null);

  // ui state
  const [busy, setBusy] = useState<boolean>(false);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    ping()
      .then((r) => setHealthy(r.ok))
      .catch(() => setHealthy(false));
  }, []);

  // When user clicks "Calculate"
  async function handleCalculate(file: File): Promise<void> {
    setBusy(true);
    setErr(null);

    try {
      const csv = await readCsvFile(file);

      // get summary stats
      const s = await statsSummaryFromCsv(csv);
      setSummary(s);

      // get distribution data (histograms etc.)
      const d = await statsDistributionFromCsv(csv);
      setDist(d);

      // once ECDF / QQ endpoints exist, wire them here
      setEcdf(null);
      setQq(null);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : "Calculation failed";
      setErr(msg);
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="min-h-screen flex flex-col bg-gradient-to-br from-[#f8f8fa] via-[#f4f4f5] to-[#efefef] text-neutral-900">
      {/* Minimal nav */}
      <NavBar />

      <main className="flex-1 px-4 py-10 flex flex-col items-center gap-8">
        {/* Analyzer panel */}
        <div className="w-full flex flex-col items-center gap-3">
          <AnalyzePanel busy={busy} error={err} onCalculate={handleCalculate} />

          {/* Health sits just under the panel */}
          <div className="text-center text-[11px] text-neutral-500 leading-snug">
            <div className="inline-flex items-center gap-2">
              <span className="text-neutral-600 font-medium">
                Backend status
              </span>
              <HealthBadge healthy={healthy} />
            </div>
            <div className="mt-1 text-neutral-400">
              {new Date().toLocaleString()}
            </div>
          </div>
        </div>

        {/* Results card */}
        <section className="rounded-2xl border border-neutral-200/60 bg-white/70 backdrop-blur-sm shadow-[0_24px_60px_-12px_rgba(0,0,0,0.12)] ring-1 ring-black/[0.03] p-6 w-full max-w-5xl">
          <header className="mb-4 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3">
            <div className="space-y-1 text-center sm:text-left">
              <h2 className="text-sm font-semibold text-neutral-900">
                Results
              </h2>
              <p className="text-xs text-neutral-600 leading-relaxed">
                Summary table, distribution plots, ECDF, & QQ diagnostics.
              </p>
            </div>

            <div className="flex items-center justify-center sm:justify-end gap-2 text-[11px] text-neutral-500">
              {busy ? (
                <span className="inline-flex items-center gap-1">
                  <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-neutral-500" />
                  Calculating…
                </span>
              ) : summary || dist ? (
                <span className="inline-flex items-center gap-1 text-emerald-600">
                  <span className="h-1.5 w-1.5 rounded-full bg-emerald-500" />
                  Ready
                </span>
              ) : (
                <span className="inline-flex items-center gap-1">
                  <span className="h-1.5 w-1.5 rounded-full bg-neutral-400" />
                  Waiting for data
                </span>
              )}
            </div>
          </header>

          <ResultBoard summary={summary} dist={dist} ecdf={ecdf} qq={qq} />
        </section>
      </main>

      <footer className="border-t border-neutral-200 bg-white/80 backdrop-blur-sm">
        <div className="mx-auto max-w-7xl px-4 h-14 flex items-center justify-between text-[11px] text-neutral-500">
          <div className="leading-tight">
            <div className="font-medium text-neutral-700">
              © {new Date().getFullYear()} Stats Utility
            </div>
            <div className="text-neutral-500">
              Rust ⟷ Python ⟷ Node.js prototype
            </div>
          </div>
          <div className="text-[10px] text-neutral-400 leading-tight text-right">
            Internal build • not production
          </div>
        </div>
      </footer>
    </div>
  );
}
