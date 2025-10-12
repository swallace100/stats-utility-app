import express from "express";
import multer from "multer";
import path from "node:path";
import fs from "node:fs/promises";
import { MongoClient, ObjectId } from "mongodb";

import { UploadJobInput, type TUploadResponse } from "@your-scope/contracts";
import { registerDocs } from "./docs";

// ----- Env & constants -----
const cwd = process.cwd();

const {
  PORT = "8080",
  MONGO_URL = "mongodb://mongo:27017/stats",
  RUST_SVC_URL = "http://stats_rs:9000",
  PLOTS_PY_URL = "http://plots_py:7000",
} = process.env;

const UPLOAD_DIR = process.env.UPLOAD_DIR || path.join(cwd, "data", "uploads");
const PLOTS_DIR = process.env.PLOTS_DIR || path.join(cwd, "data", "plots");

const NO_DB = process.env.NO_DB === "1"; // skip Mongo, use in-memory jobs
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

const nowISO = () => new Date().toISOString();

(async () => {
  // ----- Ensure dirs exist -----
  await fs.mkdir(UPLOAD_DIR, { recursive: true });
  await fs.mkdir(PLOTS_DIR, { recursive: true });

  // ----- DB (Mongo or in-memory) -----
  let Jobs: any;

  if (NO_DB) {
    const Mem: Record<string, JobDoc> = {};
    Jobs = {
      insertOne: async (doc: JobDoc) => {
        Mem[doc.jobId] = doc;
      },
      updateOne: async (q: { jobId: string }, u: { $set: Partial<JobDoc> }) => {
        const j = Mem[q.jobId];
        if (j) Mem[q.jobId] = { ...j, ...u.$set };
      },
      find: () => ({
        sort: () => ({
          limit: () => ({
            toArray: async () =>
              Object.values(Mem).sort((a, b) =>
                a.createdAt > b.createdAt ? -1 : 1,
              ),
          }),
        }),
      }),
      findOne: async (q: { jobId: string }) => Mem[q.jobId] || null,
    };
    console.log("[server] Using in-memory DB (NO_DB=1)");
  } else {
    const mongo = new MongoClient(MONGO_URL);
    await mongo.connect();
    const db = mongo.db(); // default database name from URL
    Jobs = db.collection("jobs");
    console.log("[server] Connected to Mongo");
  }

  // ----- Express app -----
  const app = express();
  app.disable("x-powered-by");
  app.use(express.json({ limit: "10mb" }));

  // Health
  app.get("/health", (_req, res) => res.json({ ok: true }));

  // Static files (so results can return public URLs)
  app.use("/files/uploads", express.static(UPLOAD_DIR));
  app.use("/files/plots", express.static(PLOTS_DIR));

  // Docs (Swagger UI using your generated OpenAPI)
  registerDocs(app);

  // ----- Multer (multipart/form-data) for /upload -----
  const storage = multer.diskStorage({
    destination: (_req, _file, cb) => cb(null, UPLOAD_DIR),
    filename: (_req, file, cb) =>
      cb(null, `${new ObjectId().toHexString()}_${file.originalname}`),
  });
  const upload = multer({ storage });

  // POST /upload  (fields: file, metadata)
  app.post("/upload", upload.single("file"), async (req, res) => {
    try {
      const metaStr = (req.body?.metadata ?? "").toString();
      const parsed = UploadJobInput.safeParse(
        metaStr ? JSON.parse(metaStr) : {},
      );
      if (!parsed.success) {
        return res.status(400).json({ error: parsed.error.flatten() });
      }
      const { kind, params } = parsed.data;

      const savedFilename = req.file?.filename;
      if (!savedFilename)
        return res.status(400).json({ error: "file is required" });
      const filePath = path.join(UPLOAD_DIR, savedFilename);

      // Create job
      const jobId = new ObjectId().toHexString();
      const jobDoc: JobDoc = {
        jobId,
        kind,
        status: "queued",
        filePath,
        params: params ?? {},
        createdAt: nowISO(),
        updatedAt: nowISO(),
      };
      await Jobs.insertOne(jobDoc);

      // async processing (fire-and-forget)
      processJob(jobDoc).catch((err) => console.error("processJob error", err));

      const resp: TUploadResponse = { jobId, status: "queued" };
      return res.status(202).json(resp);
    } catch (e: any) {
      return res.status(500).json({ error: String(e?.message || e) });
    }
  });

  // GET /jobs
  app.get("/jobs", async (_req, res) => {
    const items = await Jobs.find().sort({ createdAt: -1 }).limit(50).toArray();
    const out = items.map((i: JobDoc) => ({
      jobId: i.jobId,
      kind: i.kind,
      status: i.status,
      createdAt: i.createdAt,
      updatedAt: i.updatedAt,
      error: i.error,
      result: i.result,
    }));
    res.json(out);
  });

  // GET /results/:jobId
  app.get("/results/:jobId", async (req, res) => {
    const { jobId } = req.params as { jobId: string };
    const job: JobDoc | null = await Jobs.findOne({ jobId });
    if (!job) return res.status(404).json({ error: "Not found" });

    // Optional: map file system path to public URL
    // const publicUrl =
    //   (job.result as any)?.imagePath?.startsWith(PLOTS_DIR)
    //     ? (job.result as any).imagePath.replace(PLOTS_DIR, "/files/plots")
    //     : (job.result as any)?.imagePath;

    res.json({
      jobId: job.jobId,
      status: job.status,
      result: job.result, // or { ...(job.result as any), publicUrl }
      error: job.error,
    });
  });

  // ----- listen -----
  app.listen(Number(PORT), "0.0.0.0", () => {
    console.log(`API listening on port ${PORT}`);
  });

  // ----- async worker -----
  async function processJob(job: JobDoc) {
    await Jobs.updateOne(
      { jobId: job.jobId },
      { $set: { status: "running", updatedAt: nowISO() } },
    );

    try {
      if (FAKE_SERVICES) {
        const fake =
          job.kind === "stats"
            ? { mean: 28.3, variance: 42.0 }
            : {
                imagePath: path.join(PLOTS_DIR, "fake_plot.png"),
                meta: { chart: "hist", col: "age" },
              };

        await Jobs.updateOne(
          { jobId: job.jobId },
          { $set: { status: "succeeded", result: fake, updatedAt: nowISO() } },
        );
        return;
      }

      if (job.kind === "stats") {
        const r = await fetch(`${RUST_SVC_URL}/compute`, {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({ filePath: job.filePath, ...job.params }),
        });
        const data = await r.json();
        await Jobs.updateOne(
          { jobId: job.jobId },
          { $set: { status: "succeeded", result: data, updatedAt: nowISO() } },
        );
      } else if (job.kind === "plot") {
        const r = await fetch(`${PLOTS_PY_URL}/plot`, {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({ filePath: job.filePath, ...job.params }),
        });
        const data = await r.json(); // e.g., { imagePath, meta }
        await Jobs.updateOne(
          { jobId: job.jobId },
          { $set: { status: "succeeded", result: data, updatedAt: nowISO() } },
        );
      } else {
        throw new Error(`Unknown kind: ${job.kind}`);
      }
    } catch (e: any) {
      await Jobs.updateOne(
        { jobId: job.jobId },
        {
          $set: {
            status: "failed",
            error: String(e?.message || e),
            updatedAt: nowISO(),
          },
        },
      );
    }
  }
})();
