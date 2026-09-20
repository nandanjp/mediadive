# ADR-0007 — Generated API contracts

**Status:** Accepted · 2026-09-15 · amended 2026-09-20

## Context

Shared types between backend and frontend were an explicit developer-experience
requirement. Types alone are not enough: `as Version` on a parsed response is an
unchecked assertion, so an unexpected body surfaces as an undefined field deep in
a component rather than as an error at the boundary.

## Decision

`utoipa` derives OpenAPI from Axum handlers and DTOs. **`@hey-api/openapi-ts`**
then generates, from that one document, both the TypeScript types and the
matching **zod schemas**. Responses are parsed through the schema in
`web/src/lib/api.ts`. All generated artifacts are committed, and CI regenerates
them and fails on any diff.

Types and schemas only — the generator's SDK and vendored client plugins are not
used. Requests go through a thin helper instead, so validation is visible at the
call site and Next's per-request caching stays under our control.

## Consequences

- One source of truth: types and runtime validation are generated from the same
  spec, so they cannot disagree. Hand-written zod beside generated types would
  drift, which is why it was rejected.
- A wrong-shaped response fails at the boundary with field-level errors rather
  than rendering as `undefined`.
- Error bodies are parsed through the generated `Problem` schema, so
  `docs/API.md`'s stable `code` is typed. Parsing is best-effort — a failure from
  ingress or a proxy is not problem+json, and still yields a usable error.
- `utoipa` captures endpoints, not just payload shapes, so paths, methods, status
  codes and error bodies are all in the document.
- A committed OpenAPI document is diffable, so breaking changes are visible in
  review before the frontend discovers them.
- **The CI drift check is the point of the decision, not a nicety.** A contract
  that can drift silently is not a contract.
- The generator's toolchain is pinned to TypeScript 5.9 inside
  `packages/api-types`, isolated from the web app's TypeScript 7. TypeScript 7
  restructured the compiler API and the generator ecosystem has not caught up;
  isolating it lets the application stay current without waiting.
- Cost: annotation burden on handlers and DTOs, and a generated directory in the
  tree. Accepted.
