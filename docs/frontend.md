# Frontend (React + Vite + Tailwind + shadcn/ui)

A minimal UI to upload a CSV, calculate stats (summary + distribution), and display results/plots from the backend.

- Framework: React + Vite
- Styling: Tailwind + shadcn/ui
- API base: `VITE_API_URL`
- Local dev port (Vite): `5173`
- Container port: Nginx serves at `80` → mapped to host `8085` in Docker Compose
- Backend (in Compose): exposed on host `8080`

---

## Quick Start (Local Dev)

```bash
# from apps/frontend
npm install
npm run dev
# Vite dev server → http://localhost:5173
```

Optional override of API base URL (the frontend will default to `http://localhost:8080` if unset):

```bash
# apps/frontend/.env.local
VITE_API_URL=http://localhost:8080
```

Production-style build:

```bash
npm run build
npm run preview   # http://localhost:4173
```

---

## Running via Docker Compose

From the repo root (or from `docker/` with Docker Desktop running):

```bash
make up
# or:
docker compose -f docker/docker-compose.yml up -d --build
```

Then open the UI in your browser at:

- Frontend: <http://localhost:8085>
- Backend health: <http://localhost:8080/health>
- Rust stats svc: <http://localhost:9000/api/v1/health>
- Python plots svc: <http://localhost:7000/health>

In Compose, the frontend image is built with:

```dockerfile
ARG VITE_API_URL=http://backend:8080
```

So inside Docker, the browser code will call `http://backend:8080/...`, which works because `backend` is the service name on the internal `appnet` network. From your _host machine_, you still use `http://localhost:8080`.

---

## What the UI Does

- Lets you choose a CSV file
- Click **Calculate**
- Sends that CSV to the backend
- Shows:
  - numeric summary stats
  - distribution data / plots
  - (future) ECDF and QQ diagnostics

The main API calls:

- `POST /analyze/summary` → summary stats
- `POST /analyze/distribution` → histogram / distribution info

The backend may also proxy plot images from `/plot/*` routes.

---

## Key Files

```txt
apps/frontend/
  src/
    App.tsx
    components/
      NavBar.tsx
      AnalyzePanel.tsx
      ResultBoard.tsx
    lib/
      api.ts        # typed fetch helpers, CSV helpers
      utils.ts
```

### `App.tsx` state

- `healthy`: backend health badge
- `summary`, `dist`, `ecdf`, `qq`: analysis outputs
- `busy`: disables UI during calculation
- `err`: surfaced error message
- `handleCalculate(file)`: reads CSV → calls `/analyze/summary` and `/analyze/distribution` on the backend → updates state → `ResultBoard` renders

### `AnalyzePanel.tsx`

- Central panel at the top of the app
- File picker + "Calculate" button
- Inline tips (file size, numeric columns only, etc.)
- All text sized for readability and includes a little accent color

### `ResultBoard.tsx`

- Displays the results returned by the backend
- Shows “Waiting for data / Calculating… / Ready” states

---

## `lib/api.ts` essentials

```ts
// Base URL for all requests.
// During local dev: defaults to http://localhost:8080
// In container build: baked in as http://backend:8080
export const API_URL = import.meta.env.VITE_API_URL || "http://localhost:8080";

// Convenience: read a File object into a CSV string
export async function readCsvFile(file: File): Promise<string> {
  const text = await file.text();
  return text;
}

// Send CSV (text/plain or text/csv) to backend and get stats
export async function statsSummaryFromCsv(csv: string) {
  /* ... */
}
export async function statsDistributionFromCsv(csv: string) {
  /* ... */
}

// Optional helpers for plot PNGs
// plotSummaryPng, plotDistributionPng, etc. (backend proxies /plot/*)
```

---

## Environment Variables

| Name           | Example                 | Purpose                                                               |
| -------------- | ----------------------- | --------------------------------------------------------------------- |
| `VITE_API_URL` | `http://localhost:8080` | Backend base URL baked into frontend **at build time** (not runtime!) |

In dev (`npm run dev`), you can just point `VITE_API_URL` at `http://localhost:8080`.

In Docker Compose, the frontend image is built with `VITE_API_URL=http://backend:8080` so that the browser inside the container can reach the backend service by name on the internal bridge network.

---

## Dev Proxy (optional, for local dev without CORS pain)

Instead of hardcoding `VITE_API_URL`, you can have Vite proxy API calls:

```ts
// apps/frontend/vite.config.ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      "/health": "http://localhost:8080",
      "^/(analyze|plot|upload)": "http://localhost:8080",
    },
  },
});
```

If you do that:

- You can call `fetch("/health")` etc. directly from the browser in dev.
- You can omit `VITE_API_URL` in `.env.local` (or set it to empty).

Both approaches are valid.

---

## Troubleshooting

### Frontend shows “DOWN”

The “Backend status: DOWN” badge usually means the health check failed.

Check:

```bash
curl -fsS http://localhost:8080/health
```

If that fails in Compose, check backend logs:

```bash
make logs-one SERVICE=backend
```

### Wrong API URL

If the frontend container was built with the wrong `VITE_API_URL`, you’ll see network errors in DevTools like `GET http://localhost:8080/analyze/...` blocked or 404 from a different host.

Fix:

- Update the build arg / .env
- Rebuild just the frontend:

  ```bash
  docker compose -f docker/docker-compose.yml build frontend
  docker compose -f docker/docker-compose.yml up -d frontend
  ```

### Plots not appearing

The UI can request plot PNGs. Those get generated by the `plots_py` container and served back through the backend. Make sure:

```bash
curl -fsS http://localhost:7000/health
```

returns something OK.

---

## Accessibility / UX Notes

- The Calculate button disables while busy (`Calculating…`) so you don’t double-submit.
- Errors are shown under the button.
- File input clearly labeled “Select CSV file”.
- Panel text is readable (base 14–16px) and includes tips inline so you don’t have to hunt around the UI.

---

## Future Enhancements

- ECDF and QQ endpoints wired through backend
- Local caching of last run in `localStorage`
- Drag-and-drop CSV instead of plain `<input type="file">`
- Tabbed `ResultBoard` (Summary / Distribution / Diagnostics)
- Dark mode theme toggle
