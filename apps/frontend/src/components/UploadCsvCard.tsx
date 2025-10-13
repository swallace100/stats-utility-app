import Papa from "papaparse";
import * as React from "react";

import CSVPreview from "./CSVPreview";

import { uploadCsv, type UploadKind } from "@/lib/api";

type Cell = string;
type ArrayRow = readonly Cell[];
type ObjectRow = Readonly<Record<string, Cell>>;
type Row = ArrayRow | ObjectRow;

type UploadCsvCardProps = {
  // eslint-disable-next-line no-unused-vars
  onAnalyzeFile?: (_file: File) => void | Promise<void>;
};

export default function UploadCsvCard({ onAnalyzeFile }: UploadCsvCardProps) {
  const [file, setFile] = React.useState<File | null>(null);
  const [headers, setHeaders] = React.useState<string[] | null>(null);
  const [rows, setRows] = React.useState<Row[]>([]);
  const [kind, setKind] = React.useState<UploadKind>("stats");
  const [params, setParams] = React.useState<string>("{}");
  const [submitting, setSubmitting] = React.useState(false);
  const [result, setResult] = React.useState<{ jobId: string } | null>(null);
  const [error, setError] = React.useState<string | null>(null);

  function onPick(f: File | null) {
    setFile(f);
    setHeaders(null);
    setRows([]);
    setResult(null);
    setError(null);

    if (!f) return;

    // ✅ actively use the file
    console.log(`Picked file: ${f.name}`);
    onAnalyzeFile?.(f); // ✅ triggers the callback if provided

    // Try headered parse first
    Papa.parse<ObjectRow>(f, {
      header: true,
      dynamicTyping: false,
      worker: false,
      skipEmptyLines: "greedy",
      complete: (res) => {
        if (res.errors?.length) {
          Papa.parse<ArrayRow>(f, {
            header: false,
            dynamicTyping: false,
            worker: false,
            skipEmptyLines: "greedy",
            complete: (res2) => {
              setHeaders(null);
              setRows(res2.data ?? []);
            },
          });
          return;
        }
        const data = res.data ?? [];
        const first = data[0] || {};
        setHeaders(Object.keys(first));
        setRows(data);
      },
    });
  }

  async function onSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError(null);
    setResult(null);

    if (!file) {
      setError("Please choose a CSV file first.");
      return;
    }

    // ✅ trigger analysis before or after upload
    if (onAnalyzeFile) {
      try {
        void onAnalyzeFile(file); // call safely, ignore returned promise
      } catch {
        console.warn("onAnalyzeFile failed, continuing upload");
      }
    }

    let parsedParams: Record<string, unknown> = {};
    try {
      parsedParams = params.trim() ? JSON.parse(params) : {};
    } catch {
      setError("Params must be valid JSON.");
      return;
    }

    try {
      setSubmitting(true);
      const resp = await uploadCsv({ file, kind, params: parsedParams });
      setResult({ jobId: resp.jobId });
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Upload failed.");
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <div className="max-w-3xl p-6 rounded-2xl border border-neutral-200 bg-white shadow-sm">
      <h2 className="text-xl font-semibold">Upload CSV</h2>
      <p className="text-sm text-neutral-600 mb-4">
        Pick a CSV to preview the first 20 rows and send to the backend.
      </p>

      <form onSubmit={onSubmit} className="space-y-4">
        <div className="flex items-center gap-3">
          <input
            type="file"
            accept=".csv,text/csv"
            onChange={(e) => onPick(e.currentTarget.files?.[0] ?? null)}
            className="block w-full text-sm file:mr-3 file:py-2 file:px-3 file:rounded-md file:border file:bg-muted file:text-foreground file:hover:bg-muted/80"
          />
          <a
            href="/sample.csv"
            download
            className="text-sm underline underline-offset-4"
          >
            Download sample.csv
          </a>
        </div>

        <div className="flex flex-wrap gap-4 items-center">
          <label className="text-sm">Kind</label>
          <select
            value={kind}
            onChange={(e) => setKind(e.target.value as UploadKind)}
            className="rounded-md border bg-background px-3 py-2 text-sm"
          >
            <option value="stats">stats</option>
            <option value="plot">plot</option>
          </select>

          <label className="text-sm">Params (JSON)</label>
          <input
            value={params}
            onChange={(e) => setParams(e.target.value)}
            placeholder="{}"
            className="min-w-[240px] flex-1 rounded-md border bg-background px-3 py-2 text-sm font-mono"
          />
        </div>

        <CSVPreview headers={headers} rows={rows} />

        <div className="flex items-center gap-3">
          <button
            type="submit"
            disabled={!file || submitting}
            className={`inline-flex items-center rounded-lg px-4 py-2 text-sm font-medium ${
              !file || submitting
                ? "bg-neutral-200 text-neutral-500 cursor-not-allowed"
                : "bg-black text-white hover:opacity-90"
            }`}
          >
            {submitting ? "Uploading..." : "Send"}
          </button>

          {result && (
            <span className="text-sm">
              Queued job: <span className="font-semibold">{result.jobId}</span>
            </span>
          )}
          {error && <span className="text-sm text-red-600">{error}</span>}
        </div>
      </form>
    </div>
  );
}
