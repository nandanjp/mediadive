# ADR-0009 — Feature flag provider

**Status:** Open · 2026-09-15

## Context
Follow, voting and comments ship dark behind flags, and flags gate both backend
and frontend with per-user evaluation. OpenFeature is settled as the vendor-
neutral interface so the provider stays swappable. The provider itself is not.

OpenFeature's Rust SDK is less mature than its JavaScript and Go counterparts, so
provider support must be **verified rather than assumed**.

## Candidates
- **Flipt** — self-hosted, lightweight, fits the homelab.
- **Unleash** — self-hosted, mature, heavier.
- **GrowthBook** — self-hosted, experimentation-oriented.
- **Statsig** — managed free tier; an external dependency.

## Decision
Deferred to **milestone 8**, pending confirmation of Rust provider support. The
engine's only consumers are the flagged capabilities in milestone 9 onward, so
building it earlier means infrastructure with nothing to gate.

## Consequences
- Code targets the OpenFeature SDK, so the choice is reversible.
- Until resolved, no provider-specific API may appear outside a thin adapter.
