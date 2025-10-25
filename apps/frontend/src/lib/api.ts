// lib/api.ts

export const API_URL = import.meta.env.VITE_API_URL || "http://localhost:8080";

// ---------- health ----------
export async function ping(): Promise<{ ok: boolean }> {
  const r = await fetch(`${API_URL}/health`).catch(() => null);
  return r?.ok ? { ok: true } : { ok: false };
}

// ---------- queued upload flow ----------
export type UploadKind = "stats" | "plot";

export async function uploadCsv(opts: {
  file: File;
  kind: UploadKind;
  params?: Record<string, unknown>;
}): Promise<{ jobId: string; status: "queued" }> {
  const form = new FormData();
  form.append("file", opts.file);
  form.append(
    "metadata",
    JSON.stringify({ kind: opts.kind, params: opts.params ?? {} }),
  );

  const res = await fetch(`${API_URL}/upload`, { method: "POST", body: form });
  if (!res.ok) {
    const text = await res.text().catch(() => "");
    throw new Error(text || `Upload failed (${res.status})`);
  }
  return (await res.json()) as { jobId: string; status: "queued" };
}

// ---------- instant analyze flow (types) ----------
export type SummaryOut = {
  count: number;
  mean?: number | null;
  median?: number | null;
  std?: number | null;
  min?: number | null;
  max?: number | null;
  iqr?: number | null;
  mad?: number | null;
};

export type DistOut = {
  counts: number[];
  edges: number[];
  quantiles: [number, number][];
  skewness?: number | null;
  excess_kurtosis?: number | null;
  entropy_bits?: number | null;
};

export type EcdfOut = { xs: number[]; ps: number[] };

export type QqOut = {
  sample_quantiles: number[];
  theoretical_quantiles: number[];
  mu_hat: number;
  sigma_hat: number;
};

// ---------- CSV utils ----------
export async function readCsvFile(file: File): Promise<string> {
  const buf = await file.arrayBuffer();
  return new TextDecoder("utf-8").decode(buf);
}

export function csvToNumbers(csv: string): number[] {
  return csv
    .split(/[\n,]/g)
    .map((s) => s.trim())
    .filter(Boolean)
    .map(Number)
    .filter((x) => Number.isFinite(x));
}

// ---------- typed fetch helpers ----------
async function fetchJson<T>(path: string, init: RequestInit): Promise<T> {
  const res = await fetch(`${API_URL}${path}`, init);
  if (!res.ok) throw new Error(`${path} failed (${res.status})`);
  return (await res.json()) as T;
}

// PNG helper: strictly typed payload (object) and string result
async function postPng<B extends object>(
  path: string,
  body: B,
): Promise<string> {
  const res = await fetch(`${API_URL}${path}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!res.ok) throw new Error(`${path} failed (${res.status})`);
  const blob = await res.blob();
  return URL.createObjectURL(blob);
}

// ---------- stats (via backend) ----------
export async function statsSummaryFromCsv(csv: string): Promise<SummaryOut> {
  return fetchJson<SummaryOut>("/analyze/summary", {
    method: "POST",
    headers: { "Content-Type": "text/csv" },
    body: csv,
  });
}

export async function statsDistributionFromCsv(csv: string): Promise<DistOut> {
  return fetchJson<DistOut>("/analyze/distribution", {
    method: "POST",
    headers: { "Content-Type": "text/csv" },
    body: csv,
  });
}

export async function statsEcdfFromCsv(csv: string) {
  const r = await fetch(`${API_URL}/analyze/ecdf`, {
    method: "POST",
    headers: { "content-type": "text/csv" },
    body: csv,
  });
  if (!r.ok) throw new Error("ecdf failed");
  return r.json();
}

export async function statsQqFromCsv(csv: string) {
  const r = await fetch(`${API_URL}/analyze/qq`, {
    method: "POST",
    headers: { "content-type": "text/csv" },
    body: csv,
  });
  if (!r.ok) throw new Error("qq failed");
  return r.json();
}

// ---------- plots (via backend → returns blob URL) ----------
export function plotSummaryPng(
  summary: SummaryOut,
  title?: string,
): Promise<string> {
  const q = title ? `?title=${encodeURIComponent(title)}` : "";
  return postPng(`/plot/summary${q}`, summary);
}

export function plotDistributionPng(
  dist: DistOut,
  title?: string,
): Promise<string> {
  const q = title ? `?title=${encodeURIComponent(title)}` : "";
  return postPng(`/plot/distribution${q}`, dist);
}

export function plotEcdfPng(ecdf: EcdfOut, title?: string): Promise<string> {
  const q = title ? `?title=${encodeURIComponent(title)}` : "";
  return postPng(`/plot/ecdf${q}`, ecdf);
}

export function plotQqPng(qq: QqOut, title?: string): Promise<string> {
  const q = title ? `?title=${encodeURIComponent(title)}` : "";
  return postPng(`/plot/qq${q}`, qq);
}
