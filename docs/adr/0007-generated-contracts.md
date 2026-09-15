# ADR-0007 — Generated API contracts

**Status:** Accepted · 2026-09-15

## Context
Shared types between backend and frontend were an explicit developer-experience
requirement. The alternative considered was `ts-rs`, which exports type shapes
via derive macros.

## Decision
`utoipa` derives OpenAPI from Axum handlers and DTOs; `openapi-typescript`
generates the TypeScript client into `packages/api-types`. Both artifacts are
committed, and CI regenerates them and fails on any diff.

## Consequences
- `utoipa` captures endpoints — paths, methods, status codes, error bodies — not
  only payload shapes, so the frontend gets a typed client rather than typed
  structs it still has to wire up.
- A committed OpenAPI document is diffable, so breaking changes are visible in
  review before the frontend discovers them.
- A contract that can drift silently is not a contract; the CI check is the point
  of the decision, not a nicety.
- Cost: annotation burden on handlers and DTOs. Accepted.
