import { useEffect, useMemo, useState } from "react";

import type { SummaryOut, DistOut, EcdfOut, QqOut } from "../lib/api";
import {
  plotSummaryPng,
  plotDistributionPng,
  plotEcdfPng,
  plotQqPng,
} from "../lib/api";

type Props = {
  summary: SummaryOut | null;
  dist: DistOut | null;
  ecdf: EcdfOut | null;
  qq: QqOut | null;
};

export default function ResultBoard({ summary, dist, ecdf, qq }: Props) {
  const [summaryUrl, setSummaryUrl] = useState<string | null>(null);
  const [histUrl, setHistUrl] = useState<string | null>(null);
  const [ecdfUrl, setEcdfUrl] = useState<string | null>(null);
  const [qqUrl, setQqUrl] = useState<string | null>(null);
  const [active, setActive] = useState<"summary" | "hist" | "ecdf" | "qq">(
    "summary",
  );

  // Generate images when inputs change
  useEffect(() => {
    let url: string | null = null;
    (async () => {
      if (summary) url = await plotSummaryPng(summary);
      setSummaryUrl(url);
    })();
    return () => {
      if (url) URL.revokeObjectURL(url);
    };
  }, [summary]);

  useEffect(() => {
    (async () => {
      if (dist) setHistUrl(await plotDistributionPng(dist, "Histogram"));
      else setHistUrl(null);
    })();
  }, [dist]);

  useEffect(() => {
    (async () => {
      if (ecdf) setEcdfUrl(await plotEcdfPng(ecdf, "ECDF"));
      else setEcdfUrl(null);
    })();
  }, [ecdf]);

  useEffect(() => {
    (async () => {
      if (qq) setQqUrl(await plotQqPng(qq, "QQ Plot"));
      else setQqUrl(null);
    })();
  }, [qq]);

  const tabs = useMemo(
    () =>
      [
        { key: "summary", label: "Summary", url: summaryUrl },
        { key: "hist", label: "Histogram", url: histUrl },
        { key: "ecdf", label: "ECDF", url: ecdfUrl },
        { key: "qq", label: "QQ", url: qqUrl },
      ] as const,
    [summaryUrl, histUrl, ecdfUrl, qqUrl],
  );

  return (
    <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
      {/* Left: Stats grid */}
      <div className="lg:col-span-1">
        <div className="rounded-2xl border border-neutral-200 bg-white shadow-sm p-4">
          <h2 className="text-lg font-semibold mb-3">Stats</h2>
          {summary ? (
            <dl className="grid grid-cols-2 gap-3 text-sm">
              <Stat label="n" value={summary.count} />
              <Stat label="mean" value={summary.mean} />
              <Stat label="median" value={summary.median} />
              <Stat label="sd" value={summary.std} />
              <Stat label="min" value={summary.min} />
              <Stat label="max" value={summary.max} />
              <Stat label="IQR" value={summary.iqr} />
              <Stat label="MAD" value={summary.mad} />
            </dl>
          ) : (
            <p className="text-sm text-muted-foreground">
              Upload data to see stats.
            </p>
          )}
        </div>
      </div>

      {/* Right: Plots with tabs */}
      <div className="lg:col-span-2">
        <div className="rounded-2xl border bg-white/60 backdrop-blur shadow-sm">
          <div className="flex items-center gap-2 border-b px-3 py-2">
            {tabs.map((t) => (
              <button
                key={t.key}
                onClick={() => setActive(t.key)}
                className={`px-3 py-1.5 text-sm rounded-full border
                    ${
                      active === t.key
                        ? "bg-black text-white border-black"
                        : t.url
                          ? "bg-white text-neutral-700 border-neutral-200 hover:bg-neutral-100"
                          : "bg-neutral-100 text-neutral-400 border-neutral-200 cursor-not-allowed"
                    }`}
                disabled={!t.url}
                title={!t.url ? "Unavailable" : t.label}
              >
                {t.label}
              </button>
            ))}
          </div>
          <div className="p-4">
            {tabs.map((t) =>
              active === t.key ? (
                t.url ? (
                  <img
                    key={t.key}
                    src={t.url}
                    alt={t.label}
                    className="mx-auto max-h-[520px] w-auto rounded-lg border"
                  />
                ) : (
                  <div className="h-64 grid place-items-center text-sm text-neutral-500">
                    {t.label} not available
                  </div>
                )
              ) : null,
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

function Stat({ label, value }: { label: string; value?: number | null }) {
  const text =
    value === null || value === undefined
      ? "—"
      : Number.isFinite(value)
        ? Number(value).toPrecision(5)
        : "—";
  return (
    <div className="rounded-lg border p-3 bg-white">
      <dt className="text-xs uppercase tracking-wide text-neutral-500">
        {label}
      </dt>
      <dd className="mt-1 font-medium">{text}</dd>
    </div>
  );
}
