# mediadive

Media tracking and discovery — a catalogue of anime, drama and film with per-user
libraries, ratings, reviews and curated lists. Rust API, Next.js web app,
self-hosted on k3s.

**The documents in `docs/` are the specification.** If code and docs disagree,
one of them is a bug — say so rather than guessing which.

## Where things are defined

| Question | Document |
|---|---|
| What a word means, what must always be true | `docs/CONTEXT.md` |
| Who can do what, what is out of scope | `docs/REQUIREMENTS.md` |
| What the system is, and why | `docs/ARCHITECTURE.md` + `docs/adr/` |
| How the eight non-trivial paths work | `docs/FLOWS.md` |
| What is stored | `docs/DATA-MODEL.md` |
| API conventions — errors, pagination, casing | `docs/API.md` |
| What is tested and how | `docs/TESTING.md` |
| How code ships, and in what order | `docs/DELIVERY.md` |
| Current milestone checklist | `docs/milestones/` |
| One-time cluster setup | `deploy/BOOTSTRAP.md` |

## Standing constraints

These are easy to violate months later, and two of the three are enforced by CI.

1. **Migrations are backward-compatible** — expand/contract, always. Both colours
   of a blue-green rollout run against one database.
2. **Outbox handlers are idempotent.** Delivery is at-least-once.
3. **Cache keys carry a schema version.** Both colours share one Redis.

## Rules

- **Never commit the age private key.** It is installed in-cluster by hand and
  never enters git. The repository is public: anything committed by mistake is
  **rotated**, not merely deleted.
- **Never push to `main`.** It is protected and auto-deploys; work goes through a
  pull request.
- sqlx row types are never serialized to the API. Row → domain → DTO.
- Generated artifacts are committed and CI fails on drift. After changing a DTO
  or a handler, run `just contracts`.
- After changing any SQL, run `just prepare` — the workspace compiles against
  committed `.sqlx` metadata.

## Commands

`just` is for humans; CI calls `cargo` and `pnpm` directly.

```
just up          # local dependencies
just migrate     # apply migrations (run before `cargo check`)
just check       # fmt + clippy, as CI runs them
just test        # rust test suite
just contracts   # regenerate OpenAPI + TypeScript
just prepare     # refresh .sqlx after changing SQL
just images      # build all three container images
```
