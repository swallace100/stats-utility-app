# Docker & Compose

This stack runs the React frontend, Node backend, Rust stats service, Python plotting service, and supporting databases via Docker Compose.

- Compose file location: `./docker/docker-compose.yml`
- Default network: `appnet` (bridge)
- Persistent volumes: `db_data`, `mongo_data`, `plot_cache`, `pgadmin_data`, `data`

## Quick Start

```bash
# from ./docker
docker compose --env-file ../.env config # sanity-check variable resolution
docker compose up -d --build # build & start everything
docker compose ps # list services/ports
```

Open:

- Frontend: <http://localhost:8085>
- Backend: <http://localhost:8080/health>
- Stats RS: <http://localhost:9000/api/v1/health>
- Plots PY: <http://localhost:7000/health>
- Postgres: localhost:5432 (inside net: db:5432)
- pgAdmin: <http://localhost:5050>
- Mongo: localhost:27017 (inside net: mongo:27017)
- Mongo Express: <http://localhost:8081> (bound to 127.0.0.1 only)

## Topology

```mermaid
flowchart LR
FE[frontend (nginx)] -->|HTTP| BE[backend (Node)]
BE --> RS[stats_rs (Rust)]
BE --> PY[plots_py (FastAPI)]
BE --> PG[(Postgres)]
BE --> MG[(MongoDB)]
PGAD[pgadmin4] --> PG
MEX[mongo-express] --> MG
subgraph Volumes
V1[(db_data)]:::vol
V2[(mongo_data)]:::vol
V3[(plot_cache)]:::vol
V4[(pgadmin_data)]:::vol
V5[(data)]:::vol
end
classDef vol fill:#f6f8fa,stroke:#c9d1d9,color:#57606a
```

## Services

| Service           | Build/Image                                                            | Ports                 | Healthcheck                                | Depends on                                                            |
| ----------------- | ---------------------------------------------------------------------- | --------------------- | ------------------------------------------ | --------------------------------------------------------------------- |
| **frontend**      | `../apps/frontend/Dockerfile` (ARG `VITE_API_URL=http://backend:8080`) | `8085:80`             | `curl http://127.0.0.1/`                   | backend (started)                                                     |
| **backend**       | `apps/backend/Dockerfile`                                              | `8080:8080`           | `curl http://127.0.0.1:8080/health`        | db (healthy), mongo (healthy), stats_rs (started), plots_py (started) |
| **stats_rs**      | `../apps/stats_rs/Dockerfile`                                          | `9000:9000`           | `curl http://127.0.0.1:9000/api/v1/health` | —                                                                     |
| **plots_py**      | `../apps/plots_py/Dockerfile` (target `runtime`)                       | `7000:7000`           | `curl http://127.0.0.1:7000/health`        | —                                                                     |
| **db**            | `postgres:16`                                                          | _(internal 5432)_     | `pg_isready`                               | —                                                                     |
| **pgadmin**       | `dpage/pgadmin4:9.8`                                                   | `5050:80`             | `wget http://127.0.0.1/`                   | db (healthy)                                                          |
| **mongo**         | `mongo:7` (`--auth`)                                                   | `27017:27017`         | `mongosh ... ping`                         | —                                                                     |
| **mongo_express** | `mongo-express:1`                                                      | `127.0.0.1:8081:8081` | `wget http://127.0.0.1:8081/`              | mongo (healthy)                                                       |

Healthchecks gate depends_on for db and mongo to ensure the backend waits for ready databases.

## Environment

Compose reads from `../.env`. Required vars (used in this file):

### Common service URLs (used by backend)

- `RUST_SVC_URL` → e.g. `http://stats_rs:9000/api/v1`
- `PLOT_SVC_URL` → e.g. `http://plots_py:7000`

### Postgres

- `POSTGRES_HOST=db`
- `POSTGRES_PORT=5432`
- `POSTGRES_USER`
- `POSTGRES_PASSWORD`
- `POSTGRES_DB`

Backend composes DATABASE_URL like:

```bash
postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@${POSTGRES_HOST}:${POSTGRES_PORT}/${POSTGRES_DB}
```

### Mongo

- `MONGO_HOST=mongo`
- `MONGO_PORT=27017`
- `MONGO_INITDB_ROOT_USERNAME`
- `MONGO_INITDB_ROOT_PASSWORD`

Backend composes `MONGO_URL` like:

```bash
mongodb://${MONGO_INITDB_ROOT_USERNAME}:${MONGO_INITDB_ROOT_PASSWORD}@${MONGO_HOST}:${MONGO_PORT}/?authSource=admin
```

### pgAdmin

- `PGADMIN_DEFAULT_EMAIL`
- `PGADMIN_DEFAULT_PASSWORD`

### Backend toggles

- `NO_DB=1` (skip DB access; good for local swagger-only mode)
- `FAKE_SERVICES=1` (mock downstreams during dev)
- `PORT=8080`
- `UPLOAD_DIR=/app/data/uploads`
- `PLOTS_DIR=/app/data/plots`

### Service-specific

- stats_rs: `DATA_DIR=/data` (mounted from `data:`)
- plots_py: `DATA_DIR=/data`, `PLOTS_DIR=/data/plots`

Tip: Make sure any app code writing to `PLOTS_DIR` aligns with mounted volumes; `plots_py` also mounts `plot_cache` at `/app/out` for static serving—use one canonical path in app code or bind the two if needed.

## Volumes

| Volume         | Mounted at                                          | Used by                        |
| -------------- | --------------------------------------------------- | ------------------------------ |
| `db_data`      | `/var/lib/postgresql/data`                          | db                             |
| `mongo_data`   | `/data/db`                                          | mongo                          |
| `pgadmin_data` | `/var/lib/pgadmin`                                  | pgadmin                        |
| `plot_cache`   | `/app/out`                                          | plots_py (render cache/static) |
| `data`         | `/app/data` (backend) · `/data` (stats_rs/plots_py) | backend, stats_rs, plots_py    |

## Smoke Tests

After `docker compose up -d`:

```bash
# healthchecks
curl -fsS http://localhost:8080/health
curl -fsS http://localhost:9000/api/v1/health
curl -fsS http://localhost:7000/health

# simple render (plots_py)
curl -fsS -X POST http://localhost:7000/render \
 -H 'content-type: application/json' -d '[1,2,3,4]' \
 --output /tmp/plot.png && file /tmp/plot.png

# stats ecdf (stats_rs)
curl -fsS -X POST http://localhost:9000/api/v1/stats/ecdf \
 -H 'content-type: application/json' \
 -d '{"values":[1,2,3,4],"max_points":1000}' | jq
```

Frontend should be available at <http://localhost:8085>
and call backend at <http://backend:8080> (baked via `VITE_API_URL` build ARG).

## Common Compose Commands

```bash
# logs
docker compose logs -f backend
docker compose logs -f db mongo stats_rs plots_py

# rebuild one service
docker compose build backend && docker compose up -d backend

# restart a service
docker compose restart plots_py

# stop / remove
docker compose down
docker compose down -v # also remove volumes (⚠️ data loss)
```

## Local Dev Tips

- Frontend: `VITE_API_URL`(build arg) points at internal service name `backend:8080`.
- Backend: set `NO_DB=1` and `FAKE_SERVICES=1` during early dev; unset for full flow.
- DB Admin:
  - pgAdmin auto-loads `./pgadmin/servers.json` (read-only bind)
  - Mongo Express is restricted to localhost on port 8081 via `127.0.0.1:8081:8081`
- Health gates: backend waits for `db` + `mongo` to be healthy; `stats_rs` + `plots_py` just need to be started.

## Troubleshooting

- Backend can’t reach DB/Mongo
  - Check `.env` values: `POSTGRES_HOST=db`, `MONGO_HOST=mongo`.
  - `docker compose exec backend env | grep -E 'POSTGRES|MONGO'`
  - Verify health: `docker compose logs -f db mongo`
- plots not appearing
  - Confirm `plots_py` health and that the app writes to a mounted path (`PLOTS_DIR=/data/plots` vs `/app/out`).
  - Check cache volume: `docker compose exec plots_py ls -l /app/out`
- frontend API errors (CORS or 502)
  - Make sure frontend was built with correct `VITE_API_URL` (defaults to <http://backend:8080> inside the network; external consumers hit backend at <http://localhost:8080>).
- port conflicts
  - Change host ports in `ports:` (left side), e.g. `8086:80` for frontend.

## Notes on Security

- Databases are exposed o localhost for dev convenience; in production, avoid publishing ports and rely on an internal network.
- Mongo Express and pgAdmin are dev tooling; protect with strong creds and restrict exposure (already 127.0.0.1 for ME).

## CI/Smoke Example

```bash
docker compose up -d --build
docker compose exec -T stats_rs curl -fsS http://127.0.0.1:9000/api/v1/health
docker compose exec -T plots_py curl -fsS http://127.0.0.1:7000/health
docker compose exec -T backend curl -fsS http://127.0.0.1:8080/health
```
