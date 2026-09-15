# ADR-0005 — Identity in-process, not a separate auth service

**Status:** Accepted · 2026-09-15

## Context
A separate auth service was considered on the grounds that it would be more
secure.

## Decision
`identity` is a crate with its own schema inside the API binary.

## Consequences
- The security argument for splitting is blast-radius isolation, which is largely
  notional when both processes share a Postgres instance and a cluster network.
- The vulnerabilities that actually compromise auth are in-module concerns:
  Argon2 parameters, rate limiting and lockout, session fixation, OAuth state and
  PKCE handling, cookie flags, CSRF, account enumeration. Process separation
  fixes none of them.
- Splitting would reintroduce inter-service trust — tokens, signing keys,
  revocation propagation — i.e. the complexity ADR-0003 removes.
- Roles, blocking and moderation act on the account table, which stays local and
  transactional with the rest of the domain.
- Owning its schema keeps a later move to a deployed IdP (Keycloak, Ory, Zitadel)
  available.
