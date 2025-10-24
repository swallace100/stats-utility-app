# Backend (Node.js + Express)

The backend orchestrates uploads, job tracking, and proxy calls to:

- 🦀 `stats_rs` (numeric/statistics)
- 🐍 `plots_py` (PNG plots)

It also serves Swagger docs and static files for uploaded CSVs and rendered images.

- Port: `8080`
- Base URL: `/`
- Docs: `/docs`, `/openapi.json` (served by `src/docs.ts`)

## Endpoints

### Health

- `GET /health` → `{"ok": true}`

### Upload & Jobs

- `POST /upload` (multipart/form-data)
  Fields:
- `file`: CSV file
- `metadata`: JSON string matching `UploadJobInput` → `{ kind: "stats"|"plot", params?: object }`
  Resp: `202 Accepted` → `{ jobId: string, status: "queued" }`
- `GET /jobs` → Recent jobs (max 50), newest first
- `GET /results/{jobId}` → `{ jobId, status, result?, error? }`

### Analyze (proxies to stats_rs)

CSV body; content-type may be `text/csv`, `text/plain`, or `application/octet-stream`.
`POST /analyze/summary` → `SummaryOut`
`POST /analyze/distribution` → `DistOut`

### Plot (proxies to plots_py → returns PNG)

JSON body (mirrors `plots_py` Pydantic models). Optional `?title=....`

- `POST /plot/summary` → `image/png`
- `POST /plot/distribution` → `image/png`
- `POST /plot/ecdf` → `image/png`
- `POST /plot/qq` → `image/png`

### Static files

- `GET /files/uploads/*` → saved CSVs

- `GET /files/plots/*` → saved PNGs

## OpenAPI & Swagger

- Raw schema: `GET /openapi.json`
- UI: `GET /docs` (Swagger UI with `explorer: true`)

The OpenAPI document (in `src/docs.ts`) summarizes all routes above.

## Example CLI

```bash
# Health
curl -fsS http://localhost:8080/health | jq

# Upload (multipart): CSV + metadata
curl -fsS -X POST http://localhost:8080/upload \
  -F "file=@./sample.csv" \
  -F 'metadata={"kind":"stats","params":{}}' \
  | jq

# List jobs
curl -fsS http://localhost:8080/jobs | jq

# Get a job result
curl -fsS http://localhost:8080/results/<JOB_ID> | jq

# Analyze → summary (CSV body)
curl -fsS -X POST http://localhost:8080/analyze/summary \
  -H 'content-type: text/csv' \
  --data-binary $'value\n1\n2\n3\n4\n5' | jq

# Analyze → distribution (CSV body)
curl -fsS -X POST http://localhost:8080/analyze/distribution \
  -H 'content-type: text/csv' \
  --data-binary $'value\n1\n2\n3\n4\n5' | jq

# Plot → summary (JSON body → PNG)
curl -fsS -X POST "http://localhost:8080/plot/summary?title=Summary" \
  -H 'content-type: application/json' \
  -d '{"count":5,"mean":3,"median":3,"std":1.58,"min":1,"max":5,"iqr":2,"mad":1.48}' \
  --output summary.png
```

## Build & Run

### Local (Node)

```bash
# from apps/backend
npm install
npm run dev # or: npm start (depends on your scripts)
# server: 0.0.0.0:8080
```

Ensure required env vars (below) are set or provided via `.env`.

### Docker

Image is built by docker/Dockerfile referenced in compose as apps/backend/Dockerfile.

```bash
# from repo root (or docker/)
docker compose -f docker/docker-compose.yml build backend
docker compose -f docker/docker-compose.yml up -d backend
```

### Compose

Backend depends on:

- `stats_rs` (started)
- `plots_py` (started)

Health check: `GET /health`.

## Environment

Taken from `process.env` (`.env` loaded by Compose):

| Var             | Default                | Usage                                                  |
| --------------- | ---------------------- | ------------------------------------------------------ |
| `PORT`          | `8080`                 | HTTP port                                              |
| `RUST_SVC_URL`  | `http://stats_rs:9000` | Base for stats service                                 |
| `PLOTS_PY_URL`  | `http://plots_py:7000` | Base for plot service                                  |
| `UPLOAD_DIR`    | `<cwd>/data/uploads`   | Disk path for uploaded CSVs                            |
| `PLOTS_DIR`     | `<cwd>/data/plots`     | Disk path for saved PNGs                               |
| `FAKE_SERVICES` | `0`                    | When `1`, bypass Rust/Python and return canned results |

In production, prefer `FAKE_SERVICES=0` and the Compose-mounted volume `data:`.

## Internals & Flow

- CSV ingestion: `multer` stores the upload under `UPLOAD_DIR` with a generated ObjectId prefix.
- Async worker: `processJob(job)` reads the saved CSV and:
  - `kind="stats"` → POST CSV to `${RUST_SVC_URL}/api/v1/stats/summary`, saves JSON result
  - `kind="plot"` → POST CSV to `${PLOTS_PY_URL}/render-csv`, saves PNG to `PLOTS_DIR`, adds `publicUrl: /files/plots/<jobId>.png`
- Proxy helpers:
  - `fetchJSON(url, init)` → JSON with status check
  - `fetchPNG(url, init)` → Buffer with status check
- Static serving: `/files/uploads` and /`files/plots` map disk paths to public URLs

## Error Handling

The backend returns:

- `200 OK` (JSON or PNG) on success
- `400 Bad Request` for invalid input (e.g., empty CSV body, missing file)
- `502 Bad Gateway` if downstream Rust/Python calls fail
- `404 Not Found` for missing `jobId`
- `5xx` for unexpected internal errors

CSV routes explicitly reject empty bodies with `400`.

## Contracts (shape & stability)

- Analyze JSON mirrors `stats_rs` outputs: `SummaryOut`, `DistOut`
- Plot JSON mirrors `plots_py` inputs: `SummaryOut`, `DistOut`, `EcdfOut`, `QqOut`, `CorrMatrixOut`, `SeriesWithOutliers`
- Extend contracts additively to avoid breaking consumers
- Routes are stable under the same paths to keep frontend clients working

## Tests

(Recommended) Add integration tests under `apps/backend/tests` using `supertest` or `vitest`:

- Health
- Analyze happy/error paths
- Plot happy/error paths (assert `image/png`)
- Upload → Jobs → Results lifecycle

## Security Notes

- This service is a proxy—validate and cap inputs:
  - CSV size limits (`10mb` already configured)
  - Timeouts on fetch to Rust/Python (consider adding)
- Static files are served from controlled directories only
- For public deployments, add:
  - CORS configuration
  - Request timeouts & rate-limits
  - Auth on `/upload`, `/jobs`, `/results/*` if needed

## Troubleshooting

- 502 from /analyze or /plot → check downstream health:
  - `curl -fsS http://localhost:9000/api/v1/health` (stats_rs)
  - `curl -fsS http://localhost:7000/health` (plots_py)
- Images not visible → ensure `PLOTS_DIR` exists and file is under `/files/plots static root
- CORS during dev → run frontend via Vite proxy or enable CORS middleware here
