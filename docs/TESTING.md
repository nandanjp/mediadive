# Testing Strategy

## Levels

| Level | Scope | Dependencies |
|---|---|---|
| Unit | Mapping, validation, allowlists, cursor encoding | none |
| Integration | Handlers and repositories, one per documented flow | real Postgres and Redis |
| Index | Meilisearch behaviour specifically | real Meilisearch |
| Smoke | Promotion gate, run against production | the deployed stack |

## The rule

**Every flow in [FLOWS.md](./FLOWS.md) has at least one integration test that
walks it end to end.** Coverage percentages reward testing getters; a flow count
means something, and the document tells you when a ninth flow has appeared.

## Isolation

`#[sqlx::test]` creates a **fresh database per test** from a template, migrates
it, and drops it afterwards. The cheaper alternative — one shared database with a
transaction rolled back per test — cannot test anything requiring a real commit,
which rules out the outbox, concurrency, and `FOR UPDATE SKIP LOCKED`. Those are
the parts most worth testing.

## Dependencies

- **Postgres and Redis are real**, as CI service containers. Faking them would
  mean faking SQL.
- **Meilisearch sits behind a trait** with an in-memory implementation for most
  tests, plus a small suite against a real instance for index behaviour.
- **AniList and TMDB are never called from CI.** The upstream client is a trait;
  tests run against **JSON fixtures captured once from the real APIs**, which
  double as documentation of the actual upstream payload shapes.

## Fixtures

The five seed titles are also the integration corpus. One set of fixtures serves
both, so tests exercise the real import mapping rather than hand-built rows that
could never occur in production.

## Smoke tests

They gate promotion and run against production, which constrains them: **fast,
idempotent, non-polluting**. They assert health and readiness, a read of a known
seeded title, an authenticated round-trip using a **dedicated smoke account**
whose writes are confined to itself, and that the deployed contract version
matches what CI built.

Budget: under a minute. Slower than that and promotion becomes painful enough to
skip, which defeats the gate.

## Backward-compatibility check

Expand/contract is otherwise enforced by memory alone, and it is the constraint
most likely to be violated by accident months from now.

**The check: apply this commit's migrations to a fresh database, then run the
previously deployed api *image* against it and assert the smoke suite still
passes.** If the old binary still works, the migration is genuinely
backward-compatible. If not, blue-green would have broken production — and it
fails in CI instead.

It has to be the published image rather than the previous commit's test suite:
`#[sqlx::test]` creates a database per test and applies *that checkout's*
migrations, so an old test run would never see the new schema. The image, pointed
at an already-migrated database, reproduces the blue-green condition exactly —
old code, new schema.

The job reads the deployed revision from `deploy/charts/mediadive/values.yaml`.
While that tag is `unset` nothing has been deployed, so the check logs that it is
inert and passes. It becomes real alongside published images and the smoke suite.

This is the same thinking as the contract drift check: a rule nobody can silently
violate.

## Frontend

**Typecheck, lint and build only. No browser test suite.** A Playwright suite over
sign-in, add-to-library and write-a-review would be more machinery than this
application justifies.

The trade is explicit: backend regressions on those paths are caught by the
integration and smoke suites, which exercise them at the API level. Pure UI
regressions — a broken layout, a handler wired to nothing — are caught by using
the app. Accepted.
