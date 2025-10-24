import express from "express";
import multer from "multer";
import path from "node:path";
import fs from "node:fs/promises";
import crypto from "node:crypto";

import { UploadJobInput, type TUploadResponse } from "@your-scope/contracts";
import { registerDocs } from "./docs";

// ----- Env & constants -----
const cwd = process.cwd();

const {
  PORT = "8080",
  RUST_SVC_URL = "http://stats_rs:9000",
  PLOTS_PY_URL = "http://plots_py:7000",
} = process.env;

const UPLOAD_DIR = process.env.UPLOAD_DIR || path.join(cwd, "data", "uploads");
const PLOTS_DIR = process.env.PLOTS_DIR || path.join(cwd, "data", "plots");

const FAKE_SERVICES = process.env.FAKE_SERVICES === "1"; // don't call Rust/Python

type JobStatus = "queued" | "running" | "succeeded" | "failed";

type JobDoc = {
  jobId: string;
  kind: "stats" | "plot";
  status: JobStatus;
  filePath: string;
  params: Record<string, unknown>;
  createdAt: string;
  updatedAt: string;
  error?: string;
  result?: unknown;
};

// util
const nowISO = () => new Date().toISOString();
const newJobId = () => crypto.randomUUID();

// helpers – small wrappers around fetch for JSON and PNG proxying
async function fetchJSON(url: string, init?: RequestInit) {
  const r = await fetch(url, init);
  if (!r.ok) throw new Error(`${url} -> ${r.status}`);
  return r.json();
}

async function fetchPNG(url: string, init?: RequestInit) {
  const r = await fetch(url, init);
  if (!r.ok) throw new Error(`${url} -> ${r.status}`);
  const ab = await r.arrayBuffer();
  return Buffer.from(ab);
}

// -----------------------------
// In-memory job storage
// -----------------------------
const Mem: Record<string, JobDoc> = {};

function insertJob(doc: JobDoc) {
  Mem[doc.jobId] = doc;
}

function updateJob(jobId: string, patch: Partial<JobDoc>) {
  const j = Mem[jobId];
  if (j) {
    Mem[jobId] = { ...j, ...patch, updatedAt: nowISO() };
  }
}

function getJob(jobId: string): JobDoc | null {
  return Mem[jobId] || null;
}

function listJobs(limit = 50): JobDoc[] {
  // newest first by createdAt
  return Object.values(Mem)
    .sort((a, b) => (a.createdAt > b.createdAt ? -1 : 1))
    .slice(0, limit);
}

// -----------------------------
// main bootstrap (async IIFE so we can await fs.mkdir)
// -----------------------------
(async () => {
  // Ensure dirs exist
  await fs.mkdir(UPLOAD_DIR, { recursive: true });
  await fs.mkdir(PLOTS_DIR, { recursive: true });

  console.log("[server] Using in-memory job store (NO_DB mode)");

  // ----- Express app -----
  const app = express();
  app.disable("x-powered-by");
  app.use(express.json({ limit: "10mb" }));

  // CSV text parser for /analyze/* routes
  const textCsv = express.text({
    type: ["text/csv", "text/plain", "application/octet-stream"],
    limit: "10mb",
  });

  // Health
  app.get("/health", (_req, res) => res.json({ ok: true }));

  // Static directories so we can serve uploaded CSVs / generated plots if needed
  app.use("/files/uploads", express.static(UPLOAD_DIR));
  app.use("/files/plots", express.static(PLOTS_DIR));

  // Docs (Swagger / OpenAPI UI)
  registerDocs(app);

  // ---------- ANALYZE (stats_rs) ----------
  app.post("/analyze/summary", textCsv, async (req, res) => {
    try {
      const csv = (req.body ?? "") as string;
      if (!csv.trim()) {
        return res.status(400).json({ error: "empty CSV body" });
      }

      const out = await fetchJSON(`${RUST_SVC_URL}/api/v1/stats/summary`, {
        method: "POST",
        headers: { "content-type": "text/csv" },
        body: csv,
      });

      res.json(out);
    } catch (e: unknown) {
      const msg =
        e instanceof Error ? e.message : `unexpected error: ${String(e)}`;
      res.status(502).json({ error: msg });
    }
  });

  app.post("/analyze/distribution", textCsv, async (req, res) => {
    try {
      const csv = (req.body ?? "") as string;
      if (!csv.trim()) {
        return res.status(400).json({ error: "empty CSV body" });
      }

      const out = await fetchJSON(`${RUST_SVC_URL}/api/v1/stats/distribution`, {
        method: "POST",
        headers: { "content-type": "text/csv" },
        body: csv,
      });

      res.json(out);
    } catch (e: unknown) {
      const msg =
        e instanceof Error ? e.message : `unexpected error: ${String(e)}`;
      res.status(502).json({ error: msg });
    }
  });

  // ---------- PLOT (plots_py) → returns PNG ----------
  function pngHeaders(res: express.Response) {
    res.setHeader("content-type", "image/png");
    res.setHeader("cache-control", "no-store");
  }

  app.post("/plot/summary", async (req, res) => {
    try {
      const q = req.query?.title
        ? `?title=${encodeURIComponent(String(req.query.title))}`
        : "";
      const png = await fetchPNG(`${PLOTS_PY_URL}/plot/summary${q}`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(req.body ?? {}),
      });
      pngHeaders(res);
      res.end(png);
    } catch (e: unknown) {
      const msg =
        e instanceof Error ? e.message : `unexpected error: ${String(e)}`;
      res.status(502).json({ error: msg });
    }
  });

  app.post("/plot/distribution", async (req, res) => {
    try {
      const q = req.query?.title
        ? `?title=${encodeURIComponent(String(req.query.title))}`
        : "";
      const png = await fetchPNG(`${PLOTS_PY_URL}/plot/distribution${q}`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(req.body ?? {}),
      });
      pngHeaders(res);
      res.end(png);
    } catch (e: unknown) {
      const msg =
        e instanceof Error ? e.message : `unexpected error: ${String(e)}`;
      res.status(502).json({ error: msg });
    }
  });

  app.post("/plot/ecdf", async (req, res) => {
    try {
      const q = req.query?.title
        ? `?title=${encodeURIComponent(String(req.query.title))}`
        : "";
      const png = await fetchPNG(`${PLOTS_PY_URL}/plot/ecdf${q}`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(req.body ?? {}),
      });
      pngHeaders(res);
      res.end(png);
    } catch (e: unknown) {
      const msg =
        e instanceof Error ? e.message : `unexpected error: ${String(e)}`;
      res.status(502).json({ error: msg });
    }
  });

  app.post("/plot/qq", async (req, res) => {
    try {
      const q = req.query?.title
        ? `?title=${encodeURIComponent(String(req.query.title))}`
        : "";
      const png = await fetchPNG(`${PLOTS_PY_URL}/plot/qq${q}`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(req.body ?? {}),
      });
      pngHeaders(res);
      res.end(png);
    } catch (e: unknown) {
      const msg =
        e instanceof Error ? e.message : `unexpected error: ${String(e)}`;
      res.status(502).json({ error: msg });
    }
  });

  // Optional convenience route: plot directly from CSV
  app.post("/plot/from-csv", textCsv, async (req, res) => {
    try {
      const csv = (req.body ?? "") as string;
      if (!csv.trim()) {
        return res.status(400).json({ error: "empty CSV body" });
      }

      const png = await fetchPNG(`${PLOTS_PY_URL}/render-csv`, {
        method: "POST",
        headers: { "content-type": "text/csv" },
        body: csv,
      });

      pngHeaders(res);
      res.end(png);
    } catch (e: unknown) {
      const msg =
        e instanceof Error ? e.message : `unexpected error: ${String(e)}`;
      res.status(502).json({ error: msg });
    }
  });

  // ----- Multer (multipart/form-data) for /upload -----
  const storage = multer.diskStorage({
    destination: (_req, _file, cb) => cb(null, UPLOAD_DIR),
    filename: (_req, file, cb) => {
      const idPart = newJobId();
      cb(null, `${idPart}_${file.originalname}`);
    },
  });
  const upload = multer({ storage });

  // POST /upload  (fields: file, metadata)
  app.post("/upload", upload.single("file"), async (req, res) => {
    try {
      // metadata may describe what kind of job this is
      const metaStr = (req.body?.metadata ?? "").toString();
      const parsed = UploadJobInput.safeParse(
        metaStr ? JSON.parse(metaStr) : {},
      );
      if (!parsed.success) {
        return res.status(400).json({ error: parsed.error.flatten() });
      }

      const { kind, params } = parsed.data;

      const savedFilename = req.file?.filename;
      if (!savedFilename) {
        return res.status(400).json({ error: "file is required" });
      }

      const filePath = path.join(UPLOAD_DIR, savedFilename);

      // create new job
      const jobId = newJobId();
      const jobDoc: JobDoc = {
        jobId,
        kind,
        status: "queued",
        filePath,
        params: params ?? {},
        createdAt: nowISO(),
        updatedAt: nowISO(),
      };
      insertJob(jobDoc);

      // async background processing (fire-and-forget)
      processJob(jobDoc).catch((err) => console.error("processJob error", err));

      const resp: TUploadResponse = { jobId, status: "queued" };
      return res.status(202).json(resp);
    } catch (e: unknown) {
      const msg =
        e instanceof Error ? e.message : `unexpected error: ${String(e)}`;
      return res.status(500).json({ error: msg });
    }
  });

  // GET /jobs  -> recent jobs (in-memory)
  app.get("/jobs", (_req, res) => {
    const items = listJobs(50).map((i) => ({
      jobId: i.jobId,
      kind: i.kind,
      status: i.status,
      createdAt: i.createdAt,
      updatedAt: i.updatedAt,
      error: i.error,
      result: i.result,
    }));
    res.json(items);
  });

  // GET /results/:jobId
  app.get("/results/:jobId", (req, res) => {
    const { jobId } = req.params as { jobId: string };
    const job = getJob(jobId);
    if (!job) {
      return res.status(404).json({ error: "Not found" });
    }

    res.json({
      jobId: job.jobId,
      status: job.status,
      result: job.result,
      error: job.error,
    });
  });

  // ----- Start server -----
  app.listen(Number(PORT), "0.0.0.0", () => {
    console.log(`API listening on port ${PORT}`);
  });

  // ----- async worker -----
  async function processJob(job: JobDoc) {
    // mark running
    updateJob(job.jobId, { status: "running" });

    try {
      if (FAKE_SERVICES) {
        // fake output without calling Rust/Python services
        const fakeResult =
          job.kind === "stats"
            ? {
                count: 100,
                mean: 12.3,
                median: 12.0,
                std: 0.8,
                min: 9.0,
                max: 15.2,
                iqr: 1.1,
                mad: 0.7,
              }
            : {
                imagePath: path.join(PLOTS_DIR, "fake_plot.png"),
                publicUrl: "/files/plots/fake_plot.png",
              };

        updateJob(job.jobId, {
          status: "succeeded",
          result: fakeResult,
        });
        return;
      }

      // Read CSV contents from disk
      const csv = await fs.readFile(job.filePath, "utf-8");

      if (job.kind === "stats") {
        // ask Rust service for summary stats
        const data = await fetchJSON(`${RUST_SVC_URL}/api/v1/stats/summary`, {
          method: "POST",
          headers: { "content-type": "text/csv" },
          body: csv,
        });

        updateJob(job.jobId, {
          status: "succeeded",
          result: data,
        });
      } else if (job.kind === "plot") {
        // ask Python service to render plot image
        const png = await fetchPNG(`${PLOTS_PY_URL}/render-csv`, {
          method: "POST",
          headers: { "content-type": "text/csv" },
          body: csv,
        });

        const filename = `${job.jobId}.png`;
        const absPath = path.join(PLOTS_DIR, filename);
        await fs.writeFile(absPath, png);

        const publicUrl = `/files/plots/${filename}`;

        updateJob(job.jobId, {
          status: "succeeded",
          result: { imagePath: absPath, publicUrl },
        });
      } else {
        throw new Error(`Unknown kind: ${job.kind}`);
      }
    } catch (e: unknown) {
      const msg =
        e instanceof Error ? e.message : `unexpected error: ${String(e)}`;

      updateJob(job.jobId, {
        status: "failed",
        error: msg,
      });
    }
  }
})();
