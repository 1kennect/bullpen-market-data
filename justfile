# Bullpen tasks; just alone lists them.

set shell := ["bash", "-euo", "pipefail", "-c"]

default:
    @just --list --unsorted

# ---------- first run ----------

# Check required tools, .env, and git hooks
doctor:
    ./scripts/doctor.sh

# Create .env from the example; settings live in bullpen.toml
env:
    @[ -f .env ] && echo ".env exists" || { cp .env.example .env && echo "created .env, fill in tokens and model URLs"; }

# Install the pre-commit hook (runs the gate for touched units)
hooks:
    git config core.hooksPath .githooks
    @echo "hooks installed: .githooks/pre-commit"

# Set up a fresh clone: .env, hooks, deps
bootstrap: env hooks setup
    @echo "ready. Next a model, because the engine answers nothing without one:"
    @echo "  GPU:    just model       llama-server on CUDA"
    @echo "  no GPU: just model-cpu   the same models on the processor, slowly"
    @echo "  hosted: set BULLPEN_MODEL__BASE_URL and BULLPEN_MODEL__API_KEY in .env"
    @echo "Then 'just postgres' and 'just store', then 'just cli' for the console, or 'just engine' and 'just discord' in separate shells."

# ---------- everything ----------

# Format, lint, and test every unit
check: check-rust check-evals check-web
    @echo "all units ok"

# Format every unit in place
fmt:
    cargo fmt --all
    cd apps/evals && uvx ruff format . && uvx ruff check --fix .
    cd apps/web      && npx eslint . --fix

# Install every unit's dependencies
setup:
    cd apps/evals && uv sync --extra dev
    cd apps/web      && npm ci
    cargo fetch

# Remove build artifacts and virtualenvs
clean:
    cargo clean
    rm -rf apps/evals/.venv apps/web/node_modules apps/web/dist

# ---------- rust: engine + store + discord ----------

# Rust fmt check, clippy, tests, dependency direction
check-rust:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    ./scripts/check-deps.sh

# Run the engine
engine *ARGS:
    cargo run -p engine -- {{ARGS}}

# Run the store service; needs Postgres (just postgres)
store *ARGS:
    cargo run -p store -- {{ARGS}}

# Store tests against a real Postgres at BULLPEN_STORE_TEST_DATABASE_URL (just postgres, then a test database)
check-store-db:
    @[ -n "${BULLPEN_STORE_TEST_DATABASE_URL:-}" ] || { echo "set BULLPEN_STORE_TEST_DATABASE_URL"; exit 1; }
    cargo test -p store

# Run the discord bot
discord *ARGS:
    cargo run -p discord -- {{ARGS}}

# Developer console
cli:
    cargo run -p cli --release

# ---------- python: evals ----------

# Lint and test evals
check-evals:
    cd apps/evals && uvx ruff check . && uvx ruff format --check . && uv run --extra dev pytest -q

# Run evals
eval *ARGS:
    cd apps/evals && uv run eval {{ARGS}}

# ---------- web ----------

# Lint, typecheck, test, and build the web app
check-web:
    cd apps/web && npm run lint && npm run typecheck && npm test && npm run build

# Vite dev server
web:
    cd apps/web && npm run dev

# ---------- infra ----------

# Start the dev stack
up *ARGS:
    docker compose -f deploy/compose.yml up -d {{ARGS}}

# Stop the dev stack across every profile
down: sandbox-down
    docker compose -f deploy/compose.yml --profile model --profile metrics --profile gpu-metrics --profile phoenix down

# Remove the sandbox containers and egress proxy the engine starts outside compose
sandbox-down:
    #!/usr/bin/env bash
    set -uo pipefail
    # DOCKER_HOST from .env selects the runtime that holds them.
    [ -f .env ] && set -a && . ./.env && set +a
    held=$(docker ps -aq --filter "label=bullpen.sandbox" 2>/dev/null)
    if [ -n "$held" ]; then docker rm --force $held >/dev/null; echo "removed $(echo "$held" | wc -l) sandbox containers"; fi
    docker rm --force "${BULLPEN_SANDBOX__EGRESS_PROXY_NAME:-bullpen-sandbox-proxy}" >/dev/null 2>&1 || true
    docker network rm "${BULLPEN_SANDBOX__EGRESS_NETWORK:-bullpen-sandbox}" >/dev/null 2>&1 || true

# Start production from GHCR images
prod-up *ARGS:
    docker compose -f deploy/compose.yml -f deploy/compose.prod.yml pull
    docker compose -f deploy/compose.yml -f deploy/compose.prod.yml up -d {{ARGS}}

# Stop production
prod-down: sandbox-down
    docker compose -f deploy/compose.yml -f deploy/compose.prod.yml down

# Follow production logs
prod-logs *ARGS:
    docker compose -f deploy/compose.yml -f deploy/compose.prod.yml logs -f {{ARGS}}

# Postgres for the store on localhost:5432; requires BULLPEN_POSTGRES_PASSWORD
postgres *ARGS:
    docker compose -f deploy/compose.yml up -d {{ARGS}} postgres

# Phoenix trace UI on localhost:6006
phoenix *ARGS:
    docker compose -f deploy/compose.yml --profile phoenix up -d {{ARGS}} phoenix

# llama-server chat on the CPU
model-cpu *ARGS:
    docker compose -f deploy/compose.yml -f deploy/compose.cpu.yml --profile model up -d {{ARGS}} chat

# llama-server chat on :8000
model *ARGS:
    docker compose -f deploy/compose.yml --profile model up -d {{ARGS}} chat

# Prometheus (:9090) and Grafana (:3000); requires BULLPEN_GRAFANA_PASSWORD
metrics *ARGS:
    docker compose -f deploy/compose.yml --profile metrics up -d {{ARGS}} prometheus grafana

# GPU utilization and VRAM into Prometheus; requires NVIDIA GPU
gpu-metrics *ARGS:
    docker compose -f deploy/compose.yml --profile gpu-metrics up -d {{ARGS}} gpu-exporter

# Compose containers across every profile
ps:
    docker compose -f deploy/compose.yml --profile model --profile metrics --profile gpu-metrics --profile phoenix ps -a

# Follow dev stack logs across every profile
logs *ARGS:
    docker compose -f deploy/compose.yml --profile model --profile metrics --profile gpu-metrics --profile phoenix logs -f {{ARGS}}

# Build the rust image locally
images:
    docker build -f deploy/docker/rust.Dockerfile -t bullpen-rust .

# Build the sandbox and egress proxy images under the tags the engine runs
sandbox-images:
    docker build -f deploy/docker/sandbox.Dockerfile -t bullpen-sandbox:main .
    docker build -f deploy/docker/sandbox-proxy.Dockerfile -t bullpen-sandbox-proxy:main .

# ---------- docs ----------

# Render the ARCHITECTURE.md mermaid diagrams to verify syntax
diagrams:
    #!/usr/bin/env bash
    set -euo pipefail
    d=$(mktemp -d)
    awk -v d="$d" '/^```mermaid/{n++; f=d"/d"n".mmd"; next} /^```/{f=""; next} f{print > f}' docs/ARCHITECTURE.md
    for f in "$d"/*.mmd; do npx -y @mermaid-js/mermaid-cli -i "$f" -o "${f%.mmd}.svg" -q && echo "ok $(basename "$f")"; done
