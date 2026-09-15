# ADR-0002 — A modular monolith, not microservices

**Status:** Accepted · 2026-09-15

## Context
The predecessor project ran ten services for an application operated by one
person. Reading our own requirements: writes are rare and small, reads are mostly
anonymous and cacheable, and nothing needs independent scaling. No requirement
calls for distribution, and distribution was not a goal in itself.

## Decision
One API binary plus one worker binary, internally decomposed into a crate per
domain. One Postgres instance with a schema per domain.

## Consequences
- Cargo enforces module boundaries at compile time; the module graph cannot rot
  into a ball of mud by accident.
- No inter-service communication layer, no gRPC, no service discovery, no
  distributed tracing across process hops.
- Transactions span domains when they need to, so consistency is free.
- Three deployables means a fast, simple pipeline — which matters because CI/CD
  was a first-order requirement.
- Extracting a service later means promoting a crate to a binary and replacing
  direct calls with a client, along an already-drawn seam.
