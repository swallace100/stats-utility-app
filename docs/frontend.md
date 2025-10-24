# Frontend (React + Vite + Tailwind + shadcn/ui)

A minimal UI to upload a CSV, run quick stats (summary + distribution), and display results/plots via the backend.

- Framework: React + Vite
- Styling: Tailwind + shadcn/ui
- API base: VITE_API_URL (defaults to <http://localhost:8080>)
- Container port: Nginx serves at 80 → mapped to host 8085 in Compose

## Quick Start (Local Dev)

```bash
# from apps/frontend
npm install
npm run dev
# Vite dev server → http://localhost:5173
```

Create an env (optional; otherwise defaults to <http://localhost:8080>
):

```bash
# apps/frontend/.env.local
VITE_API_URL=http://localhost:8080
```

Prod build:

```bash
npm run build
npm run preview   # http://localhost:4173
```

Running via Docker Compose

In the repo root:

```bash
# builds all services, including frontend (served by nginx)
make up      # or: docker compose -f docker/docker-compose.yml up -d --build
```

Open: <http://localhost:8085>

The frontend is built with `VITE_API_URL=http://backend:8080` at image build time (internal service name on the Compose network).

### What the UI Does

- Pings backend health: `GET ${API_URL}/health`
- Reads a CSV file locally (`File → TextDecoder`)
- Sends CSV to backend for:
  - `POST /analyze/summary` → `SummaryOut`
  - `POST /analyze/distribution` → `DistOut`
- Shows results in `ResultBoard` (summary + distribution)
- (Placeholders for ECDF/QQ once endpoints are wired)
- Optional plotting helpers in `lib/api.ts` (`/plot/*` routes)

### Key Files

```bash
apps/frontend/
src/
    App.tsx
    components/
        NavBar.tsx
        AnalyzePanel.tsx
        ResultBoard.tsx
    lib/
        api.ts # API_URL, CSV utils, typed fetch helpers
        utils.ts
```

`App.tsx` state & flow

- `healthy`: backend health
- `summary`, `dist`, `ecdf`, `qq`: analysis outputs
- `busy`, `err`: UX flags
- `onQuickAnalyze(file)`: reads CSV → calls `/analyze/summary` & `/analyze/distribution`

`lib/api.ts` essentials

- `API_URL = import.meta.env.VITE_API_URL || "http://localhost:8080"`
- `statsSummaryFromCsv(csv)`: POST CSV → `/analyze/summary`
- `statsDistributionFromCsv(csv)`: POST CSV → `/analyze/distribution`
- Plot helpers: `plotSummaryPng`, `plotDistributionPng`, `plotEcdfPng`, `plotQqPng`
- CSV helpers: `readCsvFile`, `csvToNumbers`

### Environment Variables

| Name           | Example                 | Purpose                                 |
| -------------- | ----------------------- | --------------------------------------- |
| `VITE_API_URL` | `http://localhost:8080` | Backend base URL used at **build time** |

### Dev Proxy (optional)

To avoid CORS during local dev, you can proxy API calls in Vite:

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

Then set `VITE_API_URL=""` (empty) in `.env.local` and fetch with relative paths; or keep `VITE_API_URL` and skip the proxy—either works.

### Scripts

```bash
npm run dev       # vite dev server
npm run build     # production build (dist/)
npm run preview   # serve dist locally
npm run lint      # (if configured)
```

### Troubleshooting

- Frontend shows “DOWN”
  Backend at V`ITE_API_URL` is not reachable. Check:
  - Local: `curl -fsS http://localhost:8080/health`
  - Compose: `curl -fsS http://localhost:8080/health` and container logs (`make logs-one SERVICE=backend`)
- CORS errors in dev
  Use the Vite proxy (above) or ensure backend allows your dev origin.

- Wrong API URL after container build
  Rebuild the frontend image after changing `VITE_API_URL`:

```bash
docker compose -f docker/docker-compose.yml build frontend && docker compose up -d frontend
```

- Plots not appearing
  Backend must forward to `plots_py`. Ensure those services are healthy:

```bash
curl -fsS http://localhost:7000/health
```

### Accessibility & UX Notes

- Buttons/labels use clear contrast and text states (Analyzing…).
- File input accepts .csv,text/csv.
- Error state (err) is surfaced near the action area.

### Future Enhancements

- Wire ECDF/QQ to backend endpoints
- Persist last analysis (localStorage)
- Drag-and-drop zone for CSV
- Basic result caching (hash CSV → avoid re-compute)
- Add tests (React Testing Library) for `QuickAnalyze` happy/error paths
