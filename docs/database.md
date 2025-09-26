# Database Design

## PostgreSQL (authoritative, transactional)

- users, datasets, jobs
- results_index with summary columns

## MongoDB (flexible artifacts)

- results_blobs: full JSON output
- plot_specs: chart specs
- run_contexts: env metadata

### Why Mix?

- SQL: integrity, reporting
- Mongo: fast evolution, flexible shapes

### Cons

- Two systems to operate
- Orchestration needed (Outbox pattern, CDC)
