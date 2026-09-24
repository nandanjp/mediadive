# ADR-0010 — oxlint, and staying on TypeScript 7

**Status:** Accepted · 2026-09-20

## Context

TypeScript 7 restructured the compiler API. Three tools broke on it in a single
afternoon:

- `openapi-typescript` — `ts.factory` is gone; latest still declares `typescript@^5.x`
- `typescript-eslint` — refuses TS 7 outright; peer range is `>=4.8.4 <6.1.0`
- ESLint 10 — changed the `SourceCode` scope-manager API, which
  `@typescript-eslint/parser` has not caught up with

Keeping `eslint-config-next` working would have meant holding back **two**
majors — TypeScript 7 → 6 and ESLint 10 → 9 — to satisfy one dev tool. Both
paths were built and verified before choosing.

## Decision

**oxlint** for linting, on **TypeScript 7**. ESLint and `eslint-config-next` are
removed. The contract generator keeps its own pinned **TypeScript 5.9** inside
`packages/api-types`, isolated from the app.

## Consequences

- oxlint never touches the TypeScript compiler API, so it is immune to this
  entire class of breakage.
- Lint runs in ~0.1s rather than ~1.5s.
- **No type-aware rules.** `no-floating-promises` and `no-misused-promises` are
  the real loss, and they matter in a codebase that is async throughout. `tsc`
  still typechecks; revisit if oxlint ships type-aware analysis or
  typescript-eslint supports TS 7.
- `react/react-in-jsx-scope` is disabled — Next compiles with the automatic JSX
  runtime, making it a false positive on every component.
- `next lint` was **removed** in Next 16; the script calls `oxlint` directly.
- **Do not "restore" ESLint without re-reading this.** It will fail the same way.
