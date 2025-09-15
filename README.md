# stats-utility-app

A statistics utility apps that calculates a variable of statistics based on a user provided CSV. It is a React/Node application in a Docker container with Python and Rust based microservices for calculations.

## Architecture (React + Node.js + Rust + Python + Docker)

[React UI]
│
▼
[Node.js API] ──(RPC/HTTP/MsgQueue)──> [Rust stats svc]
│ │
│ (JSON results) └─ crunches stats (fast, deterministic)
│
└───> builds a chart spec ───────────────► [Python matplotlib svc]
(JSON in) └─ renders PNG/SVG + returns URL/bytes

## Repo Layout

stats-utility/
apps/
frontend/ # React (Vite or Next.js), Tailwind, shadcn/ui
backend/ # Node.js (TypeScript), Fastify/Express
stats_rs/ # Rust microservice (Axum or Actix)
packages/
shared-types/ # zod/io-ts schemas shared FE/BE
docker/
backend.Dockerfile
frontend.Dockerfile
stats_rs.Dockerfile
docker-compose.yml
README.md

## Example Flow (End-to-End)

1. User uploads CSV → Node stores datasetId.
2. User selects two-sample t-test → Node calls Rust /ttest with column picks.
3. Rust responds with stats JSON → Node displays table text + builds a default chart spec (e.g., violin_with_box for groups).
4. Node calls Py /render with that spec → gets PNG URL → front end shows chart alongside the stats.
5. User clicks Export → Node bundles Markdown report + tables + images.

## API Surface (Sketch)

- POST /upload → returns datasetId. (Streams to disk; infer schema.)
- POST /jobs → { datasetId, analysis: "ttest_two_sample", params: {...} } → returns jobId.
- GET /jobs/:jobId/stream → SSE for progress.
- GET /results/:jobId → JSON: stats, tables, pretty text, csv/markdown exports.

## Rust Microservice (what it owns)

- Robust CSV loader (handles headers, missing values, locale commas).
- Column typing (numeric/ordinal/categorical with thresholds).
- Numeric kernels:
  - Descriptives (O(n) single-pass where possible).
  - Tests (Welch’s t by default; Levene for variance check optional).
  - Chi-square with Yates correction toggle.
  - ANOVA (one-way; Tukey post-hoc later).
  - Simple OLS (β, SE, t, p, CI, R²) with basic residual checks.
- Deterministic JSON outputs with metadata (n, df, assumptions).

### Example Rust function signatures

```rust
    pub fn describe(x: &[f64]) -> DescribeOut { /* mean, sd, se, ci95, ... */ }
    pub fn ttest_welch(x: &[f64], y: &[f64]) -> TTestOut { /* t, df, p, ci */ }
    pub fn chisq_independence(a: &Array2<u64>) -> ChiSqOut { /* X2, df, p */ }
    pub fn ols_simple(x: &[f64], y: &[f64]) -> OlsOut { /* beta0, beta1, ... */ }
```

### Data Contracts

```json
{
  "jobId": "j_123",
  "datasetId": "d_abc",
  "analysis": "ttest_two_sample",
  "inputs": { "x": "height_cm", "y": "group" },
  "result": {
    "t": 2.153,
    "df": 38.7,
    "p": 0.0371,
    "ci": [0.8, 12.4],
    "meanX": 172.4,
    "meanY": 166.1,
    "cohenD": 0.68,
    "assumptions": { "welch": true }
  },
  "meta": { "nX": 21, "nY": 19, "missing": 2, "seed": 0 }
}
```

## Python Chart Microservice

- Framework: FastAPI (Python) for quick endpoints.
- Endpoint: POST /render takes the spec, validates (pydantic), renders with matplotlib, saves to /out, returns URL + hash.
- Repro tips:
  - Set matplotlib.use("Agg") and fix font to “DejaVu Sans”.
  - Set a global random seed for jitter/violin.
  - Respect width/height/dpi; keep default colors so reviewers recognize matplotlib output.
- Caching: SHA256 over (spec JSON normalized) → filename. If exists, return existing.

### Node -> Python (chart spec)

```json
{
  "chartId": "c_789",
  "type": "violin_with_box",
  "title": "Height by Group",
  "data": {
    "series": [
      { "name": "A", "values": [170,172,169, ...] },
      { "name": "B", "values": [163,165,168, ...] }
    ]
  },
  "enc": { "y": "values", "x": "name" },
  "style": {
    "width": 900, "height": 600,
    "dpi": 144, "font": "DejaVu Sans",
    "grid": true
  },
  "annotations": [
    { "kind": "text", "text": "Welch t=2.15, p=0.037", "xy": [0.5, 0.95], "coords": "axes" }
  ],
  "export": { "format": "png", "transparent": false }
}
```

### Python -> Node (render response)

```json
{
  "chartId": "c_789",
  "url": "http://plots:7000/render/c_789.png",
  "sha256": "ae4f...c2",
  "format": "png",
  "width": 900,
  "height": 600,
  "bytes": null
}
```

## Frontend UX

- Dataset page: preview table, type toggles, choose variables, quick data cleaning (drop NA, z-score outliers).
- Analysis wizard: pick method → choose columns → assumptions checklist → “Run”.
- Results:
  - Clean APA-style tables (copy as Markdown/HTML/CSV).
  - Auto summary (e.g., “There was a significant difference… t(38)=2.15, p=0.037, d=0.68”).
  - Downloadable report (.md or .docx later).

## Docker Contracts

```ts
export const TTestTwoSampleParams = z.object({
  xColumn: z.string(),
  yColumn: z.string(),
  equalVariances: z.boolean().default(false),
  alternative: z.enum(["two-sided", "less", "greater"]).default("two-sided"),
});
export const TTestOut = z.object({
  t: z.number(),
  df: z.number(),
  p: z.number(),
  ci: z.tuple([z.number(), z.number()]),
  meanX: z.number(),
  meanY: z.number(),
  cohenD: z.number(),
});
```

## Database
### When to mix databases (and why)
- Relational core (Postgres/MySQL): strict integrity, joins, transactions. Great for users, permissions, datasets, jobs, and audit trails.
- Document layer (MongoDB): flexible, sparse, rapidly evolving shapes (custom fields, settings, artifacts/specs) without migrations.
- (Optional) Search layer (OpenSearch/Elasticsearch): fast, fuzzy search across names, notes, and attachments.
- (Optional) Cache/queue (Redis/Kafka): speed + decoupling for ingestion and async jobs.
Many CRM/contact products follow this pattern.

### A concrete split for your Stats Utility App
PostgreSQL (authoritative, transactional)
- users, datasets, jobs (status, started_at, finished_at)
- results_index (one row per analysis result; small summary columns; pointer to full blob)
- Foreign keys, constraints, easy reporting
MongoDB (flexible artifacts & evolving schemas)
- results_blobs: the full JSON output from Rust (test stats, diagnostics), versioned
- plot_specs: the chart spec you send to the Python service (these evolve a lot)
- run_contexts: environment hashes, library versions, CPU flags (nice for reproducibility)
- (Optional) custom_fields: arbitrary tags/notes a user attaches to datasets/jobs
Why not just Postgres JSONB? You could. But using Mongo gives you practice with:
- rapidly changing doc shapes,
- partial updates,
- different indexing strategies (compound, text, TTL),
- separate scaling/backup knobs (exactly what big products do).

### How services talk without losing consistency

- Node API is the orchestrator. It writes the truth to Postgres (create job), then sends compute to Rust.
- Rust returns results → Node:
  1. stores full blob in Mongo (results_blobs), gets _id,
  2. writes a summary row to Postgres results_index (with the Mongo _id).
- For plots: Node builds a chart spec, saves it in Mongo plot_specs, calls Python to render, and stores the file path/hash back in Postgres (optional).
- If you need stronger guarantees across stores, use the Outbox pattern (write an events row in Postgres within the same tx; a worker reads and applies it to Mongo) or CDC later.

#### Pros
- Right tool for each job: strict integrity + flexible evolution.
- Performance: hot JSON docs in Mongo; clean reporting in SQL.
- Evolvability: add fields to artifacts/specs without migrations.

#### Cons
- Operational complexity (two backups, two monitoring stacks).
- No cross-DB transactions → need orchestration patterns (outbox/saga).
- Duplicate data requires discipline (clear ownership rules).

## Details

### Why add a separate plotting service?

#### Pros

- Language separation: Rust stays numeric; Python owns viz ergonomics (mpl ecosystem is rich).
- Reusability: any future service can request plots via the same API.
- Caching: image cache is independent and cheap.
- Extensibility: can add seaborn/plotnine later without touching Rust/Node.

#### Cons

- More moving parts (deploy + logs).
- Slight latency added (usually fine; renders are quick for small datasets).

### Testing & reliability

- Contract tests in Node using fixed CSVs: assert Rust’s numeric outputs (golden files).
- Snapshot tests for plots: compare sha256 of rendered PNGs for a fixed spec (ensure reproducibility).
- Load tests (small): parallel renders with different specs to validate cache behavior.

### Security & perf notes

- Validate inputs in both Node and Py (length caps, numeric ranges).
- Limit max rows plotted (e.g., downsample to 50k points) and warn user.
- Enforce timeouts in Node when calling Rust/Py; return graceful errors.
- Use content-addressable filenames so the same spec never re-renders.

### Nice default charts (map analysis → viz)

- Describe (1 column): histogram + KDE line; boxplot.
- Compare means (2 groups): violin+box, bar±CI, swarm (n≤5k).
- Categorical × categorical: mosaic/stacked bar + residuals heatmap.
- Regression: scatter + fitted line + CI band; residuals vs fitted; QQ.
