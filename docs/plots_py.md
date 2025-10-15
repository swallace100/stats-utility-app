# Rust Stats Service (`stats_rs`)

Stateless renderer that turns JSON/CSV into PNG using FastAPI + matplotlib.
Called by the Node backend; deterministic outputs for testability.

- Port: 7000
- Health/OpenAPI (when DEBUG=1): /health, /docs, /redoc, /openapi.json

- Base URL: `/api/v1`
- Health: `GET /health` → `{"ok":true}`
- OpenAPI: `GET /openapi.json`
- Content-Type: `application/json`

## Endpoints

### Describe

- `POST /api/v1/stats/describe`
  Body: `DescribeIn { values: f64[] }`
  Resp: `SummaryOut { count: usize, mean?: f64, median?: f64, std?: f64, min?: f64, max?: f64, iqr?: f64, mad?: f64 }`

### Distribution (pre-binned)

- `POST /api/v1/stats/distribution`
  Body: `DistIn { values: f64[], bins?: usize | rule?: "sturges"|"scott"|"fd"|"auto" }`
  Resp: `DistOut { counts: usize[], edges: f64[], quantiles: [f64,f64][], skewness?: f64, excess_kurtosis?: f64, entropy_bits?: f64 }`

### ECDF

- `POST /api/v1/stats/ecdf`
  Body: `EcdfIn { values: f64[], max_points?: usize }`
  Resp: `EcdfOut { xs: f64[], ps: f64[] }`

### QQ (Normal)

- `POST /api/v1/stats/qq`
  Body: `QqIn { values: f64[] }`
  Resp: `QqOut { sample_quantiles: f64[], theoretical_quantiles: f64[], mu_hat: f64, sigma_hat: f64 }`

### Correlation Matrix

- `POST /api/v1/stats/corr`
  Body: `CorrIn { columns: Record<string, f64[]> }`
  Resp: `CorrMatrixOut { size: usize, matrix: f64[], names?: string[] }`

### Two-sample t-test (Welch by default)

- `POST /api/v1/stats/ttest`
  Body: `TTestIn { x: f64[], y: f64[], equal_variances?: bool=false, alternative?: "two-sided"|"less"|"greater" }`
  Resp: `TTestOut { t: f64, df: f64, p: f64, ci: [f64,f64], meanX: f64, meanY: f64, cohenD: f64 }`

### One-way ANOVA

- `POST /api/v1/stats/anova1`
  Body: `Anova1In { groups: Record<string, f64[]> }`
  Resp: `Anova1Out { F: f64, df_between: f64, df_within: f64, p: f64 }`

### Chi-square (Independence)

- `POST /api/v1/stats/chisq`
  Body: `ChiSqIn { table: u64[][], yates?: bool=false }`
  Resp: `ChiSqOut { x2: f64, df: usize, p: f64, expected?: f64[][] }`

### Simple OLS (y ~ β0 + β1 x)

- `POST /api/v1/stats/ols_simple`
  Body: `OlsIn { x: f64[], y: f64[] }`
  Resp: `OlsOut { beta0: f64, beta1: f64, se0: f64, se1: f64, t0: f64, t1: f64, p0: f64, p1: f64, r2: f64, ci0: [f64,f64], ci1: [f64,f64] }`

### Helpers (optional)

- `POST /api/v1/stats/normalize`
  Body: `NormalizeIn { values: f64[], method: "zscore"|"minmax", range?: [f64,f64] }`
  Resp: `NormalizeOut { values: f64[] }`

- `POST /api/v1/stats/binrule`
  Body: `BinRuleIn { values: f64[], rule: "sturges"|"scott"|"fd"|"auto" }`
  Resp: `BinRuleOut { bins: usize }`

### Notes

- Optional fields (`?`) may be omitted or `null`.
- Large payloads: consider server-side caps (e.g., max N) and streaming uploads where applicable.
- All numeric arrays must be finite (`NaN/±inf` rejected).

## Example CLI

```bash
# Liveness
curl -fsS http://localhost:7000/health

# Quick render from JSON array (smoke test)
curl -fsS -X POST http://localhost:7000/render \
  -H 'content-type: application/json' \
  -d '[1,2,3,4,5]' \
  --output render.png

# Quick render from CSV (smoke test)
curl -fsS -X POST http://localhost:7000/render-csv \
  -H 'content-type: text/csv' \
  --data-binary $'val\n1\n2\n3\n4' \
  --output render_csv.png

# Summary block (Rust-shaped JSON)
curl -fsS -X POST 'http://localhost:7000/plot/summary?title=Summary' \
  -H 'content-type: application/json' \
  -d '{"count":42,"mean":1.23,"median":1.1,"std":0.56,"min":-2,"max":4.5,"iqr":0.8,"mad":0.6}' \
  --output summary.png

# Distribution (pre-binned histogram)
curl -fsS -X POST 'http://localhost:7000/plot/distribution?title=Histogram' \
  -H 'content-type: application/json' \
  -d '{"edges":[0,1,2,3,4],"counts":[5,12,9,3],"skewness":0.12,"excess_kurtosis":-0.4,"entropy_bits":1.73,"quantiles":[[0.05,0.4],[0.5,1.7],[0.95,3.1]]}' \
  --output hist.png

# ECDF
curl -fsS -X POST 'http://localhost:7000/plot/ecdf?title=ECDF' \
  -H 'content-type: application/json' \
  -d '{"xs":[1,2,3,4],"ps":[0.25,0.5,0.75,1.0]}' \
  --output ecdf.png

# QQ (Normal)
curl -fsS -X POST 'http://localhost:7000/plot/qq?title=QQ%20Normal' \
  -H 'content-type: application/json' \
  -d '{"sample_quantiles":[0.1,0.3,0.5,0.7],"theoretical_quantiles":[0.05,0.25,0.5,0.75],"mu_hat":0.02,"sigma_hat":0.98}' \
  --output qq.png

# Correlation heatmap
curl -fsS -X POST 'http://localhost:7000/plot/corr-heatmap?title=Correlation' \
  -H 'content-type: application/json' \
  -d '{"size":3,"matrix":[1,0.2,-0.1,0.2,1,0.5,-0.1,0.5,1],"names":["A","B","C"]}' \
  --output corr.png

# Series with outliers
curl -fsS -X POST 'http://localhost:7000/plot/series?title=Series' \
  -H 'content-type: application/json' \
  -d '{"values":[1.2,1.4,0.9,2.0,1.1],"outliers":{"indices":[2,4]}}' \
  --output series.png
```

## Build & Run

Local (venv)

```bash
cd apps/plots_py
python -m venv .venv
source .venv/bin/activate  # Windows: .venv\Scripts\activate
pip install -U pip
pip install -r requirements.txt

# Dev server
uvicorn main:app --host 0.0.0.0 --port 7000 --reload

# OpenAPI (when DEBUG=1)
# http://localhost:7000/docs  |  /redoc  |  /openapi.json
```

## Docker

`apps/plots_py/Dockerfile` builds a slim runtime.

- Healthcheck hits /health
- Headless matplotlib (Agg) is preconfigured

```bash
cd apps/plots_py
docker build -t plots_py:local .
docker run --rm -p 7000:7000 -e DEBUG=1 plots_py:local
```

## Compose (CI/Smoke)

`plots_py` exposes `/health` for the Compose healthcheck.

In CI, you can exec:

```bash
docker compose exec -T plots_py curl -fsS http://127.0.0.1:7000/health
```

## Error Handling

Service returns:

- `200 OK` with `image/png` for successful renders
- `400 Bad Request` for bad inputs the service detects (e.g., empty arrays, length mismatches, non-UTF-8 CSV)
- `422 Unprocessable Entity` for Pydantic validation failures (wrong shapes/types)
- `5xx` only for unexpected internal errors

Examples of explicit 400s:

- `/render`: `expected non-empty array of numbers`
- `/render-csv`: `could not decode body as utf-8`, `no numeric data found`
- `/plot/distribution`: `edges must be length counts+1`
- `/plot/ecdf`: `xs and ps must be same length`
- `/plot/qq`: sample/theoretical quantiles mismatch/non-empty check
- `/plot/corr-heatmap`: `matrix length must be size\*size`

## Features & Middleware

- Framework: FastAPI (Starlette)
- Docs (when `DEBUG=1`): `/docs`, `/redoc`, `/openapi.json`
- Rendering: `matplotlib` in Agg mode (headless), DejaVu Sans font
- Determinism: Identical inputs → identical images
- Caching: Encouraged upstream via `sha256(normalized spec)` filenames (service itself is stateless)

  (If you later add middleware, common choices are `CORSMiddleware` and`GZipMiddleware`.)

## Tests

- Location: `./apps/plots_py/tests`
- Runner: `pytest`
- Client: `fastapi.testclient.TestClient`

```bash
cd apps/plots_py
source .venv/bin/activate # if not already
pytest -q
```

### Tests cover

- `/health`
- `/render` and `/render-csv` happy paths + error cases
- Structured `/plot/*` routes (summary, distribution, ecdf, qq, corr-heatmap, series)

## Contract Notes (for backend/stats)

- Stable routes: `/render`, `/render-csv`, `/plot/`
- Request models mirror Rust outputs (`SummaryOut`, `DistOut`, `EcdfOut`, `QqOut`, `CorrMatrixOut`, `SeriesWithOutliers`)
- Keep field shapes stable; add new fields additively
- The Node backend is the orchestrator; `plots_py` is render-only (no business logic)
- Prefer content-addressable filenames upstream to avoid duplicate renders
