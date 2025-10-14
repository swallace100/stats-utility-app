# stats_rs – Rust Microservice

A small Rust/Axum service that exposes fast, deterministic statistical kernels over HTTP.
It powers the Node backend and the plots service via stable JSON APIs.

---

## Responsibilities

- Parse CSV (with/without headers), ignore non-numeric cells, tolerate ragged rows.
- Core stats: descriptives, distributions (hist/quantiles/ECDF), correlations, outliers, normalization.
- Pairwise + matrix correlations (Pearson/Spearman/Kendall).
- Deterministic, documented JSON inputs/outputs with OpenAPI at `/openapi.json`.
- Production concerns: health/readiness, compression, CORS, timeouts, graceful shutdown.

---

## HTTP Endpoints (v1)

Base: `http://<host>:9000/api/v1`

### Health

- `GET /api/v1/health` → `"ok"`
- `GET /api/v1/ready` → `"ready"`

### Describe

- `POST /api/v1/describe`
  **Body**: `{"0": [f64, ...]}` (alias: `DescribeInput` → a JSON array wrapper)
  **Resp**: `DescribeOutput { count, mean, median, std_dev }`

- `POST /api/v1/describe-csv` (`Content-Type: text/csv`)
  Parses numbers from the CSV body and returns `DescribeOutput`.

### Summary stats

- `POST /api/v1/stats/summary`
  **Body**: `SummaryIn { values: f64[] }`
  **Resp**: `SummaryOut { count, mean?, median?, std?, min?, max?, iqr?, mad? }`

### Distribution bundle

- `POST /api/v1/stats/distribution`
  **Body**: `DistIn { values: f64[], bins?: usize, quantiles?: f64[] }`
  **Resp**: `DistOut { counts: usize[], edges: f64[], quantiles: (f64,f64)[], skewness?, excess_kurtosis?, entropy_bits? }`

### Pairwise correlations (two vectors)

- `POST /api/v1/stats/pairwise`
  **Body**: `PairIn { x: f64[], y: f64[] }`
  **Resp**: `PairOut { covariance?, pearson?, spearman?, kendall? }`

### ECDF

- `POST /api/v1/stats/ecdf`
  **Body**: `EcdfIn { values: f64[], max_points?: usize }`
  **Resp**: `EcdfOut { xs: f64[], ps: f64[] }`

### QQ vs Normal

- `POST /api/v1/stats/qq-normal`
  **Body**: `QqIn { values: f64[], robust?: bool }`
  **Resp**: `QqOut { sample_quantiles: f64[], theoretical_quantiles: f64[], mu_hat: f64, sigma_hat: f64 }`

### Correlation Matrix

- `POST /api/v1/stats/corr-matrix`
  **Body**: `CorrMatrixIn { series: f64[][], names?: string[], method?: "pearson"|"spearman"|"kendall" }`
  **Resp**: `CorrMatrixOut { size: usize, names?: string[], matrix: f64[] /* row-major size*size */ }`

### Outliers

- `POST /api/v1/stats/outliers`
  **Body**: `OutliersIn { values: f64[], method?: "iqr"|"zscore", k?: f64 }`
  **Resp**: `OutliersOut { indices: usize[], values: f64[] }`

### Normalize

- `POST /api/v1/stats/normalize`
  **Body**: `NormalizeIn { values: f64[], method: "zscore"|"minmax", range?: [f64,f64] }`
  **Resp**: `NormalizeOut { values: f64[] }`

### Bin rule helper

- `POST /api/v1/stats/binrule`
  **Body**: `BinRuleIn { values: f64[], rule: "sturges"|"scott"|"fd"|"auto" }`
  **Resp**: `BinRuleOut { bins: usize }`

> **Schemas**: All request/response structs derive `serde` + `schemars`. Live JSON Schema & path docs at `/openapi.json`.
> Optional docs UI is served at `/docs` when the `docs` feature is enabled.

---

## Example CLI

```bash
# Liveness
curl -fsS http://localhost:9000/api/v1/health

# Describe JSON
curl -fsS -H 'content-type: application/json' \
  -d '{"0":[1,2,3,4,5]}' \
  http://localhost:9000/api/v1/describe

# Summary
curl -fsS -H 'content-type: application/json' \
  -d '{"values":[1,2,3,4,5]}' \
  http://localhost:9000/api/v1/stats/summary

# Distribution
curl -fsS -H 'content-type: application/json' \
  -d '{"values":[1,2,3,4,5], "bins": 4, "quantiles":[0.25,0.5,0.75]}' \
  http://localhost:9000/api/v1/stats/distribution
```

## Build & Run

### Local

```bash
cargo run
# server: 0.0.0.0:9000
```

### Docker

`apps/stats_rs/Dockerfile` builds a static-ish release and a slim runtime:

- Healthcheck hits `/api/v1/health`.
- Uses curl for healthcheck.
- Nightly toolchain can be toggled via `ARG RUST_TOOLCHAIN`.

```bash
docker build -t stats_rs:local .
docker run --rm -p 9000:9000 stats_rs:local
```

### Compose (CI/Smoke)

`stats_rs` exposes `/api/v1/health` for the healthcheck.

In CI, we exec `curl` inside the container:
`docker compose exec -T stats_rs curl -fsS http://127.0.0.1:9000/api/v1/health`

### Error Handling

Service returns:

- `200 OK` with well-typed JSON on success.
- `400 Bad Request` for invalid inputs (empty vectors, NaN/Inf, malformed CSV).
- `5xx` only for unexpected internal errors.

Rust errors are mapped via `ServiceError` → `IntoResponse`.

### Features & Middleware

Features (compile-time): `docs`, `metrics`, `rag` (optional routes).

Middleware: `TraceLayer`, `CompressionLayer`, `CorsLayer(Any)`, `TimeoutLayer(30s)`, `DefaultBodyLimit(25MB)`.

Logging: `RUST_LOG="info,axum=info,tower_http=info,hyper=warn"` (default sensible).

## Tests

Integration-style HTTP tests live in `src/tests/http.rs` using `Router.into_service().oneshot(...)`.
They cover health, describe, CSV, and the new `/stats/*` endpoints.

Run all:

```bash
cargo test
```

Contract Notes (for backend/plots)
Stable routes under `/api/v1/*` so clients don’t break.
Node backend calls `stats_rs` for numbers → passes results to `plots_py` purely for rendering.
Keep `DescribeOutput`, `SummaryOut`, and `DistOut` fields stable; extend with additive fields only.
