# ADR-0003 — Server-side sessions, not JWTs

**Status:** Accepted · 2026-09-15

## Context
JWTs with short TTLs and rotating refresh tokens exist so that services can
verify a caller without consulting a shared database. In a monolith there are no
such services. The predecessor project spent an entire specification on token
TTLs, refresh rotation and silent-refresh edge cases — and still had bugs there.

## Decision
An opaque session id in an `httpOnly`, `Secure`, `SameSite` cookie, backed by a
Postgres session table.

## Consequences
- Revocation is a `DELETE`. Blocking an account (a functional requirement) is
  immediate and total rather than eventually consistent with token expiry.
- No signing keys to generate, mount, rotate or leak.
- The entire refresh-rotation bug class disappears.
- Every authenticated request costs a session lookup — negligible at our volume,
  and the table is small and indexed.
- If services are ever extracted, they either share the session store or a token
  layer is added then, with a concrete reason.
