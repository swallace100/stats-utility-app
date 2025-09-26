export const API_URL = import.meta.env.VITE_API_URL || "http://localhost:8080";

export async function ping(): Promise<{ ok: boolean }> {
  const r = await fetch(`${API_URL}/health`);
  return r.ok ? { ok: true } : { ok: false };
}

export type UploadKind = "stats" | "plot";

export async function uploadCsv(opts: {
  file: File;
  kind: UploadKind;
  params?: Record<string, unknown>;
}) {
  const form = new FormData();
  form.append("file", opts.file);
  form.append(
    "metadata",
    JSON.stringify({ kind: opts.kind, params: opts.params ?? {} })
  );

  const res = await fetch(`${API_URL}/upload`, {
    method: "POST",
    body: form,
  });

  if (!res.ok) {
    const text = await res.text().catch(() => "");
    throw new Error(text || `Upload failed (${res.status})`);
  }
  return (await res.json()) as { jobId: string; status: "queued" };
}
