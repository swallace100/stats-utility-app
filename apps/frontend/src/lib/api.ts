export const API_URL = import.meta.env.VITE_API_URL || "http://localhost:8080";

export async function ping(): Promise<{ ok: boolean }> {
  const r = await fetch(`${API_URL}/health`);
  return r.ok ? { ok: true } : { ok: false };
}
