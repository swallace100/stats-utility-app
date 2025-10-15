# -------------------------------
# Stats Utility App - Makefile
# Location: repo root
# Requires: docker, docker compose, bash
# -------------------------------

# Paths / files
COMPOSE_FILE ?= ./docker/docker-compose.yml
ENV_FILE     ?= ./.env

# Services (from docker-compose.yml)
SERVICES := frontend backend stats_rs plots_py db pgadmin mongo mongo_express

# Default service for single-service commands (override: `make logs SERVICE=backend`)
SERVICE ?= backend

# Compose shortcut
COMPOSE := docker compose -f $(COMPOSE_FILE) --env-file $(ENV_FILE)

# Curl flags for quiet fail
CURL := curl -fsS

.PHONY: help
help: ## Show this help
	@echo "Usage: make <target> [SERVICE=name]"
	@echo ""
	@awk 'BEGIN {FS = ":.*##"; printf "Targets:\n"} /^[a-zA-Z0-9_-]+:.*##/ { printf "  \033[36m%-18s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)
	@echo ""
	@echo "Services: $(SERVICES)"
	@echo "Default SERVICE: $(SERVICE)"

# -------------------------------
# Lifecycle
# -------------------------------

.PHONY: up
up: ## Build & start all services (detached)
	$(COMPOSE) up -d --build

.PHONY: up-nobuild
up-nobuild: ## Start all services without rebuilding
	$(COMPOSE) up -d

.PHONY: up-one
up-one: ## Build & start a single service: make up-one SERVICE=backend
	$(COMPOSE) up -d --build $(SERVICE)

.PHONY: down
down: ## Stop and remove containers, keeping volumes
	$(COMPOSE) down

.PHONY: down-v
down-v: ## Stop and remove containers + volumes (⚠️ data loss)
	$(COMPOSE) down -v

.PHONY: restart
restart: ## Restart all services
	$(COMPOSE) restart

.PHONY: restart-one
restart-one: ## Restart a single service: make restart-one SERVICE=plots_py
	$(COMPOSE) restart $(SERVICE)

# -------------------------------
# Build
# -------------------------------

.PHONY: build
build: ## Build all services
	$(COMPOSE) build

.PHONY: build-one
build-one: ## Build a single service: make build-one SERVICE=frontend
	$(COMPOSE) build $(SERVICE)

# -------------------------------
# Status / Logs / Exec
# -------------------------------

.PHONY: ps
ps: ## Show container status
	$(COMPOSE) ps

.PHONY: logs
logs: ## Tail logs for all services (Ctrl+C to exit)
	$(COMPOSE) logs -f

.PHONY: logs-one
logs-one: ## Tail logs for one service: make logs-one SERVICE=backend
	$(COMPOSE) logs -f $(SERVICE)

.PHONY: exec
exec: ## Open a shell in a service: make exec SERVICE=backend
	$(COMPOSE) exec $(SERVICE) sh -lc 'test -x /bin/bash && exec /bin/bash || exec sh'

.PHONY: sh
sh: ## Alias for exec
	@$(MAKE) exec SERVICE=$(SERVICE)

# -------------------------------
# Config / Env
# -------------------------------

.PHONY: config
config: ## Print the fully-resolved compose config
	$(COMPOSE) config

.PHONY: env-check
env-check: ## Ensure required .env values exist and show a summary
	@test -f $(ENV_FILE) || (echo "ERROR: $(ENV_FILE) not found" && exit 1)
	@echo "Using $(ENV_FILE)"
	@echo "---- Extracted vars ----"
	@egrep -v '^(#|$$)' $(ENV_FILE) | sed 's/^/  /'
	@echo "------------------------"

# -------------------------------
# Health / Smoke
# -------------------------------

.PHONY: health
health: ## Hit basic health endpoints (requires services up)
	@echo "Checking backend    : http://localhost:8080/health"
	@$(CURL) http://localhost:8080/health >/dev/null && echo "  OK" || (echo "  FAIL" && exit 1)
	@echo "Checking stats_rs   : http://localhost:9000/api/v1/health"
	@$(CURL) http://localhost:9000/api/v1/health >/dev/null && echo "  OK" || (echo "  FAIL" && exit 1)
	@echo "Checking plots_py   : http://localhost:7000/health"
	@$(CURL) http://localhost:7000/health >/dev/null && echo "  OK" || (echo "  FAIL" && exit 1)

.PHONY: smoke
smoke: ## Quick end-to-end smoke checks (simple curl calls)
	@echo "→ ECDF (stats_rs)"
	@$(CURL) -X POST http://localhost:9000/api/v1/stats/ecdf \
	  -H 'content-type: application/json' \
	  -d '{"values":[1,2,3,4],"max_points":1000}' | jq '.ps' >/dev/null || exit 1
	@echo "→ Render (plots_py)"
	@$(CURL) -X POST http://localhost:7000/render \
	  -H 'content-type: application/json' \
	  -d '[1,2,3,4]' --output /tmp/plot.png && test -s /tmp/plot.png || (echo "render failed" && exit 1)
	@echo "→ Backend health"
	@$(CURL) http://localhost:8080/health >/dev/null || exit 1
	@echo "Smoke OK"

# -------------------------------
# Clean / Reset
# -------------------------------

.PHONY: clean
clean: ## Remove stopped containers and dangling images
	-docker container prune -f
	-docker image prune -f

.PHONY: reset
reset: ## Full reset: down + remove volumes + clean (⚠️ data loss)
	@$(MAKE) down-v
	@$(MAKE) clean

# -------------------------------
# Shortcuts per service
# -------------------------------

.PHONY: be fe rs py db me pgad
be:  ; @$(MAKE) logs-one SERVICE=backend   ## Tail backend logs
fe:  ; @$(MAKE) logs-one SERVICE=frontend  ## Tail frontend logs
rs:  ; @$(MAKE) logs-one SERVICE=stats_rs  ## Tail Rust service logs
py:  ; @$(MAKE) logs-one SERVICE=plots_py  ## Tail Python service logs
db:  ; @$(MAKE) logs-one SERVICE=db        ## Tail Postgres logs
me:  ; @$(MAKE) logs-one SERVICE=mongo_express ## Tail Mongo Express logs
pgad:; @$(MAKE) logs-one SERVICE=pgadmin   ## Tail pgAdmin logs
