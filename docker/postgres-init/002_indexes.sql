-- Helpful indexes
CREATE INDEX IF NOT EXISTS datasets_owner_id_idx ON datasets(owner_id);
CREATE INDEX IF NOT EXISTS jobs_dataset_id_idx ON jobs(dataset_id);
CREATE INDEX IF NOT EXISTS jobs_status_idx ON jobs(status);
CREATE INDEX IF NOT EXISTS jobs_analysis_type_idx ON jobs(analysis_type);
CREATE INDEX IF NOT EXISTS results_index_p_idx ON results_index(p_value);

-- JSONB indexes (if you’ll filter on keys)
-- Example: params->>'xColumn', params->>'yColumn'
CREATE INDEX IF NOT EXISTS jobs_params_gin ON jobs USING GIN (params);
