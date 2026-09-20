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
| Cache / rate limiting | Redis |
| Transactional email | Resend |
| Sessions | `tower-sessions`, Postgres-backed |
| Passwords / OAuth | `argon2` · `oauth2` (Google) |
| API contract | `utoipa` → OpenAPI → `@hey-api/openapi-ts` (types + zod) |
| Frontend | Next.js App Router · TipTap (rich text, inline spoiler marks) |
| Config validation | Rust: typed and validated at boot · Web: zod + t3-env |
| Client data | TanStack Query (interactive and per-user state) |
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

## Frontend data fetching

Two paths, split by who the data belongs to.

**Server components** render public, cacheable, SEO-relevant data — catalog,
media, person, profile and article pages. They call the API directly over its
internal URL.

**TanStack Query** owns interactive per-user state: library entries, ratings,
forms, anything that mutates or paginates. A query is defined once with
`queryOptions` and shared between the server prefetch and the client `useQuery`,
so a page arrives hydrated with no loading flash and no second request.

**The browser never calls the API directly.** `next.config.ts` rewrites `/api/*`
to the API, keeping browser requests same-origin so session cookies are sent —
identically in development, where web and api are on different ports, and in
production, where they share a hostname.

**Cache policy belongs on each query, not in the fetch helper.** A query with no
explicit policy is prerendered at build time — correct for a catalog page, and
wrong for anything per-user or live, which would otherwise ship with build-time
data baked in.

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
password hashed with Argon2id. Email-and-password accounts must verify before
first sign-in; Google accounts are verified on link, and link by verified email. The `identity` crate owns its schema so a move to
an external IdP stays possible.

## Deployment

```
Cloudflare edge (TLS)
  └── cloudflared (in-cluster)
        └── Traefik ingress          mediadive.nandan-hl.dev
              ├── /api/*    → api
              ├── /images/* → Garage
              └── /*        → web
```

Single k3s cluster, single node, single environment — no staging. Postgres runs
under the **CloudNativePG** operator with scheduled base backups and PITR to
Garage.

**Accepted risk:** the Postgres volume and Garage share one disk
(`/mnt/drive2`), so a drive failure loses the database and its backups together.
Deliberate — the alternative was deferring the deploy over a low-traffic
single-user system. The mitigation is that a restore is tested during bootstrap,
so the procedure is known to work; moving Garage to a second disk, or copying
dumps off the box, closes it whenever it becomes worth doing.

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

GitHub Actions on every push; Argo CD reconciles from `deploy/`. Mechanics —
branching, image tagging, promotion gating, rollback and secrets — are in
[DELIVERY.md](./DELIVERY.md).

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
6. Cache keys carry a schema version — under blue-green both colours share one Redis.
