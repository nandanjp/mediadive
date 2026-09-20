# Human convenience only. CI calls cargo and pnpm directly so the pipeline stays
# explicit — see docs/DELIVERY.md.
#
# `just` shows only the LAST comment line before a recipe, so descriptions are
# single-line and any rationale sits above a blank line.

export DATABASE_URL := env_var_or_default("DATABASE_URL", "postgres://mediadive:mediadive@localhost:5433/mediadive")

default:
    @just --list

# Create local env files from their examples.
setup:
    cp -n web/.env.example web/.env.local || true

# Start local dependencies.
up:
    docker compose up -d --wait postgres redis
    docker compose up -d meilisearch garage

# Stop local dependencies, keeping data.
down:
    docker compose down

# Destroy local data — everything here is reproducible.
reset:
    docker compose down -v

# Apply migrations. Run before `cargo check`: query! macros need a live schema.
migrate:
    sqlx migrate run --source crates/db/migrations

# One-time Garage layout. Garage refuses writes until a layout exists.
garage-init:
    #!/usr/bin/env bash
    set -euo pipefail
    id=$(docker compose exec -T garage /garage node id -q | cut -d@ -f1)
    docker compose exec -T garage /garage layout assign -z dev -c 1G "$id"
    docker compose exec -T garage /garage layout apply --version 1
    docker compose exec -T garage /garage bucket create mediadive

# Format check and lint, as CI runs them.
check:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings

# Run the Rust test suite.
test:
    cargo test --workspace

# Regenerate the API contract. CI fails if this produces a diff.
contracts:
    cargo run -q --bin openapi > packages/api-types/openapi.json
    pnpm -C packages/api-types run generate

# Refresh offline query metadata after changing any SQL.
prepare:
    cargo sqlx prepare --workspace -- --all-targets

# Build all three container images locally.
images:
    docker build --target api --build-arg MEDIADIVE_GIT_SHA=$(git rev-parse --short HEAD) -t mediadive-api:dev .
    docker build --target worker --build-arg MEDIADIVE_GIT_SHA=$(git rev-parse --short HEAD) -t mediadive-worker:dev .
    docker build -f web/Dockerfile -t mediadive-web:dev .

# Run the API against local dependencies.
api:
    cargo run --bin api

# Run the outbox worker against local dependencies.
worker:
    cargo run --bin worker
