# Milestone 0 — Skeleton

**Goal:** prove the entire pipeline before any feature exists. Commit → image →
tag bump → sync → migration hook → green → smoke → promote.

Milestone table: [DELIVERY.md](../DELIVERY.md) · Pipeline mechanics: same file.

Groups run in order; each depends on the one above.

## 1 · Repo foundation

- [x] `rust-toolchain.toml` pinning the version
- [x] Cargo workspace: `crates/{api,worker,contracts,db}`
- [x] `db`: sqlx pool, `migrations/`, first migration creates `platform.outbox`
- [x] `contracts`: one DTO (`version`) with utoipa derives
- [x] `api`: axum with `/health`, `/ready`, `/api/v1/version`, `/metrics`
- [x] `worker`: outbox claim loop using `FOR UPDATE SKIP LOCKED` — nothing to
      process yet; proves the loop
- [x] `tracing` JSON subscriber, Prometheus registry
- [x] config from env, typed and validated at boot
- [x] `web`: Next.js App Router, one page rendering `/api/v1/version`
- [x] `packages/api-types` generated from OpenAPI
- [x] web configuration validated with zod and t3-env
- [x] TanStack Query wired with server prefetch and hydration
- [x] `.sqlx` offline metadata committed

## 2 · Local dev loop

- [x] `docker-compose.yml`: Postgres, Redis, Meilisearch, Garage
- [x] `justfile`: up, migrate, test, generate-contracts
- [x] one unit test, one `#[sqlx::test]` integration test
- [x] verify contract generation is byte-stable across runs

## 3 · Containers

- [x] `api` and `worker` Dockerfiles with `cargo-chef` — deps layer, then build
- [x] `web` Dockerfile using Next standalone output
- [x] all three build locally before CI sees them

## 4 · CI — pull request pipeline

- [x] fmt · clippy `-D warnings`
- [x] offline compile proves committed `.sqlx` covers every query
- [x] `migrate` binary runs, proving what the Helm hook will run
- [x] tests against a Postgres service container
- [x] contract drift — OpenAPI in the rust job, TypeScript in the web job
- [x] web lint (oxlint), typecheck, build
- [x] all three images built and discarded
- [x] `Swatinem/rust-cache` and buildx GHA cache, shared across api and worker
- [x] concurrency group cancelling superseded runs
- [x] backward-compatibility check wired, inert until a release exists
- [x] workflow validated with `actionlint`
- [ ] branch protection on `main`: PR required, checks required — **after the
      first run**, since GitHub only allows requiring checks it has seen

Redis and Meilisearch service containers are deliberately absent: nothing tests
them yet, and unused services slow every run. They arrive with the milestone that
needs them.

## 5 · Handoff artifacts

Written before cluster work, because that runs in a **separate session with none
of this context**.

- [x] `deploy/` skeleton exists, so Argo has a real path to target
- [x] root `CLAUDE.md`: what mediadive is · map of `docs/` · the three standing
      constraints · hard rules (never commit the age private key, never push to
      `main`, docs are the spec) · pointer to the runbook
- [x] `deploy/BOOTSTRAP.md`:
  - [x] facts table to fill in and commit — nodes, disk paths and what each holds,
        app hostname, image hostname, Cloudflare account and tunnel name, k3s
        version
  - [x] pinned versions for k3s, Argo CD, Argo Rollouts, CloudNativePG, Garage,
        Meilisearch, Redis, Prometheus stack, Loki
  - [x] ordered steps, each with a verification command and expected output
  - [x] interactive steps flagged — `cloudflared tunnel login` needs a human
  - [x] failure guidance, and which steps are safe to re-run

**Handoff contract.** The bootstrap session commits back: `.sops.yaml` with the
age **public** recipient, the filled facts table, SOPS-encrypted secrets for the
stateful components, and a completion record of what was installed at what
version. It must not commit the private key, push to `main`, or touch application
code.

The age keypair is **generated on the homelab**. The public recipient travels to
the repo; the private key never travels.

## 6 · Cluster bootstrap

Hand-run once, not GitOps. Executed from `deploy/BOOTSTRAP.md`.

- [ ] k3s up, Traefik confirmed
- [ ] `cloudflared` deployment, tunnel, DNS records
- [ ] age keypair generated; private key installed as a cluster secret
- [ ] Argo CD installed; `helm-secrets` configured on the repo-server
- [ ] Argo Rollouts installed
- [ ] namespace created

## 7 · Stateful components

Into the chart. **Garage first** — Postgres backups target it.

- [ ] Garage, volume on its own disk
- [ ] CloudNativePG operator, then the Postgres cluster
- [ ] scheduled base backups and PITR targeting Garage
- [ ] Redis
- [ ] Meilisearch
- [ ] SOPS-encrypted secrets for all of the above
- [ ] verify a backup lands **and a restore works**

## 8 · Observability

- [ ] Prometheus + Grafana
- [ ] Loki + log collector
- [ ] `ServiceMonitor` for api and worker
- [ ] Grafana datasources provisioned as config, not clicked

## 9 · App chart and rollout

- [ ] Helm umbrella chart: api, worker, web
- [ ] `Rollout` with blue-green, preview service, `autoPromotionEnabled: false`
- [ ] migration Job as a `pre-upgrade` hook
- [ ] `AnalysisTemplate` running the smoke Job
- [ ] Traefik ingress routes
- [ ] Argo CD `Application` targeting `deploy/charts/mediadive`

## 10 · Smoke suite

- [ ] asserts `/health`, `/ready`, and `/api/v1/version` matching the expected SHA
- [ ] expected SHA passed into the Job from chart values

## 11 · CI — main pipeline

- [ ] build and push `ghcr.io/nandanjp/mediadive-{api,worker,web}:<sha>`
- [ ] bot commit bumping the tag in `deploy/`
- [ ] `paths-ignore: deploy/**`

## 12 · Prove it, deliberately

- [ ] a commit reaches production and promotes
- [ ] break the smoke test on purpose → abort leaves traffic on blue
- [ ] revert a commit → Argo reconciles back
- [ ] `deploy/BOOTSTRAP.md` updated with what actually happened

## Done when

A commit to `main` reaches production unattended, a failing smoke test stops it,
and a revert undoes it — with logs and metrics visible for all three.

## Open

- [ ] hostnames for the app and the image bucket
- [ ] k3s single-node or multi, and which disk holds what
- [ ] Rust version to pin
- [ ] local dependencies via compose rather than a local cluster — assumed yes
- [ ] `justfile` over `Makefile` — assumed yes
