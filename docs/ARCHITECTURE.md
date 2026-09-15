# Architecture

Requirements are in [REQUIREMENTS.md](./REQUIREMENTS.md); domain terms in
[CONTEXT.md](./CONTEXT.md). Decisions that took argument are recorded as
[ADRs](./adr/). This document describes the system as built.

## Shape

Three deployables:

| Deployable | Role |
|---|---|
| **api** | Axum HTTP service. All synchronous request handling. |
| **worker** | Drains the transactional outbox: search indexing, image processing. |
| **web** | Next.js App Router. Server-rendered public pages, client interactivity. |

## Stack

| Concern | Choice |
|---|---|
| Language / framework | Rust · Axum · tokio |
| Data access | sqlx — compile-time-checked SQL, no ORM |
| Database | PostgreSQL, one instance, schema per domain |
| Search | Meilisearch |
| Object storage | Garage (S3-compatible, self-hosted) |
| Sessions | `tower-sessions`, Postgres-backed |
| Passwords / OAuth | `argon2` · `oauth2` (Google) |
| API contract | `utoipa` → OpenAPI → `openapi-typescript` |
| Frontend | Next.js App Router · TipTap (rich text, inline spoiler marks) |
| Telemetry | `tracing` → structured JSON · Prometheus metrics |
| Feature flags | OpenFeature SDK; provider open (ADR-0009) |
| Orchestration | k3s · Helm · Argo CD · Argo Rollouts |

## Repository layout

```
mediadive/
  Cargo.toml              workspace
  crates/
    api/                  binary — routing, middleware, handlers
    worker/               binary — outbox consumer
    contracts/            DTOs, utoipa schemas, error types
    db/                   pool, migrations, row types
    domain/
      catalog/ library/ review/ list/
      article/ moderation/ identity/
  web/                    Next.js app
  packages/api-types/     generated TypeScript (CI-verified)
  deploy/
    charts/mediadive/     Helm umbrella chart
    argocd/               Application manifests
  docs/
```

Each domain is its own crate. Cargo then enforces the boundaries at compile
time — a crate cannot reach into one it does not depend on — which makes the
module graph a fact rather than a convention, and makes extracting a service
later mechanical.

## Layering

Row structs → domain types → DTOs. **sqlx row types are never serialized.** The
DTO layer in `contracts` is the only thing that crosses the network, and the only
thing exported to TypeScript. This decouples column renames from API breakage.

## Consistency

**Immediate** for anything the writing user reads next: library entries, ratings,
rating aggregates, reviews, lists. All single-transaction, all read-after-write.

**Eventual** for everything downstream of the outbox: the search index, image
variants, and any analytics rollups we materialize later.

## Asynchronous work

A write and its event row land in the same transaction. The worker claims rows
with `SELECT … FOR UPDATE SKIP LOCKED`, so delivery is at-least-once and **every
handler must be idempotent**. No broker (ADR-0004); the outbox is the seam one
would sit behind.

Meilisearch holds no authoritative state — it is rebuildable from Postgres by a
reindex job, so it needs no backup.

## Identity

Opaque session id in an `httpOnly`, `Secure`, `SameSite` cookie, backed by a
Postgres session table (ADR-0003). Blocking an account deletes its sessions.
Registration and login by Google OAuth (authorization code + PKCE) or email and
password hashed with Argon2id. The `identity` crate owns its schema so a move to
an external IdP stays possible.

## Deployment

```
Cloudflare edge (TLS)
  └── cloudflared (in-cluster)
        └── Traefik ingress
              ├── web
              └── api
```

Single k3s cluster, single environment — no staging. Postgres runs under the
**CloudNativePG** operator with scheduled base backups and PITR to Garage, whose
volume sits on a **different physical disk** from the Postgres PVC.

Manifests are a Helm umbrella chart reconciled by **Argo CD**. Secrets are
SOPS+age encrypted in-repo and decrypted at apply time.

Releases use **Argo Rollouts** blue-green: the green side is deployed and
reachable on a preview service, smoke tests run against it, and **promotion is
gated on those tests passing**. With no staging environment, the preview service
is the verification surface.

The **worker does not blue-green** — it is recreated, relying on idempotent
handlers rather than two concurrent generations.

## Migrations

Both colours run against one database during a cutover, so **every migration must
be backward-compatible**. Schema changes follow expand/contract: add, backfill and
dual-write in one release; remove the old shape in a later one. Migrations run as
a Job before promotion, never inside the application on boot.

## Pipeline

GitHub Actions, on every push:

1. `cargo fmt --check`, `cargo clippy -- -D warnings`
2. `cargo sqlx prepare --check` — committed query metadata is current
3. Unit and integration tests against Postgres and Meilisearch service containers
4. **Contract check** — regenerate OpenAPI and TypeScript; fail on any diff
5. Web typecheck, lint, build
6. Build images with `cargo-chef` layer caching; push to GHCR
7. Bot commit bumps the image tag in `deploy/charts` — guarded against retriggering
8. Argo CD syncs · migration Job · green deploy · smoke tests · promote

Rust build times are the pipeline's dominant cost; `cargo-chef` and a shared
`sccache` are load-bearing, not optimizations.

## Observability

Ships with the first release. Services emit structured JSON via `tracing` and
expose Prometheus metrics; liveness and readiness probes gate rollout promotion.
In-cluster: **Prometheus** for metrics, **Loki** for logs, **Grafana** to read
both.

## Standing constraints

1. Migrations are backward-compatible — expand/contract, always.
2. Outbox handlers are idempotent.
3. sqlx row types are never serialized to the API.
4. Committed OpenAPI and TypeScript match the code, enforced in CI.
5. The application behaves coherently with any feature flag off.
