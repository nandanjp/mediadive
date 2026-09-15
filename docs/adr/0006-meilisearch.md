# ADR-0006 — Meilisearch for search

**Status:** Accepted · 2026-09-15

## Context
Matching native-language titles — Korean, Japanese and Chinese script — is a
first-class requirement. Postgres full-text search tokenizes CJK poorly without
extensions such as `pgroonga` or `pg_bigm`, which are non-trivial to operate and
tune.

## Decision
Meilisearch, fed by the worker from the outbox.

## Consequences
- This is the one requirement that earns a dedicated piece of infrastructure.
- Typo tolerance and relevance tuning come built in rather than hand-rolled.
- Meilisearch holds no authoritative state: it is rebuildable from Postgres by a
  reindex job, so it needs no backup and a corrupt index is not an incident.
- The index lags writes by seconds — accepted, and consistent with the
  immediate/eventual split in ARCHITECTURE.md.
