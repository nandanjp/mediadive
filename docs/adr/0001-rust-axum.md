# ADR-0001 — Rust and Axum for the backend

**Status:** Accepted · 2026-09-15

## Context
The backend could be written in any language. The stated goal is to learn to
build and operate a production backend in Rust.

## Decision
Rust, with Axum on tokio. sqlx for data access — compile-time-verified SQL
against a real schema, no ORM.

## Consequences
- Compile times dominate CI. `cargo-chef` caches the dependency layer in image
  builds and `Swatinem/rust-cache` caches the registry and `target/` in
  workflows. Both are required, not optional.
- sqlx needs query metadata at build time, so `.sqlx` is committed. CI compiles
  with `SQLX_OFFLINE=true`, which fails on any query the metadata does not cover
  — no sqlx-cli needed in the pipeline. Tests then run against a real Postgres,
  so queries are also exercised against the true schema.
- sqlx teaches SQL rather than hiding it, which suits the goal.
- Ecosystem maturity is uneven at the edges — notably OpenFeature (ADR-0009).
