-- Enable UUID + good defaults
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Users
CREATE TABLE IF NOT EXISTS users (
  id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  email      TEXT UNIQUE NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Datasets: metadata only (file bytes live in volume/object store)
CREATE TABLE IF NOT EXISTS datasets (
  id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  owner_id   UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  filename   TEXT NOT NULL,
  row_count  INTEGER,
  schema     JSONB,                -- inferred columns/types summary
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Jobs
-- status: queued | running | done | error
DO $$ BEGIN
  CREATE TYPE job_status AS ENUM ('queued','running','done','error');
EXCEPTION
  WHEN duplicate_object THEN NULL;
END $$;

CREATE TABLE IF NOT EXISTS jobs (
  id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  dataset_id    UUID NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
  analysis_type TEXT NOT NULL,    -- e.g., 'describe','ttest_two_sample'
  params        JSONB NOT NULL,   -- inputs selected by user
  status        job_status NOT NULL DEFAULT 'queued',
  started_at    TIMESTAMPTZ,
  finished_at   TIMESTAMPTZ
);

-- Results index (lightweight, query-friendly)
CREATE TABLE IF NOT EXISTS results_index (
  job_id        UUID PRIMARY KEY REFERENCES jobs(id) ON DELETE CASCADE,
  p_value       DOUBLE PRECISION,
  effect_size   DOUBLE PRECISION,
  summary       TEXT,             -- APA-ish single line
  artifact_ref  TEXT,             -- Mongo _id or content hash
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);
