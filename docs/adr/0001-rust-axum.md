# ADR-0001 — Rust and Axum for the backend

**Status:** Accepted · 2026-09-15

## Context
The backend could be written in any language. The stated goal is to learn to
build and operate a production backend in Rust.

## Decision
Rust, with Axum on tokio. sqlx for data access — compile-time-verified SQL
against a real schema, no ORM.

## Consequences
- Compile times dominate CI. `cargo-chef` layer caching and a shared `sccache`
  are required, not optional.
- sqlx needs query metadata at build time: `.sqlx` is committed and CI verifies
  it with `cargo sqlx prepare --check`.
- sqlx teaches SQL rather than hiding it, which suits the goal.
- Ecosystem maturity is uneven at the edges — notably OpenFeature (ADR-0009).
