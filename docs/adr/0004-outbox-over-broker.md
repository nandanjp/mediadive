# ADR-0004 — Transactional outbox, not a message broker

**Status:** Accepted · 2026-09-15

## Context
Two things want to be asynchronous: keeping the search index in step with catalog
changes, and processing images. Nothing in the v1 requirements needs multi-
consumer fan-out — AI, recommendations, notifications and activity feeds are all
explicitly out of scope.

## Decision
Writes append an event row in the same transaction as the business change. The
worker claims rows with `SELECT … FOR UPDATE SKIP LOCKED`.

## Consequences
- No Kafka, no ZooKeeper, no consumer groups, no second data system to operate,
  back up or reason about during a failure.
- No dual-write problem: the event and the state it describes commit together.
- Delivery is at-least-once, so **every handler must be idempotent**. This is a
  hard requirement, restated in ARCHITECTURE.md.
- Throughput is bounded by Postgres, which is far above anything the
  requirements imply.
- If real fan-out appears, the outbox is the seam to put a broker behind without
  touching call sites.
