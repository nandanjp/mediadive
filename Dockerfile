# syntax=docker/dockerfile:1
#
# Rust services. Two images from one compile:
#   docker build --target api    -t mediadive-api    .
#   docker build --target worker -t mediadive-worker .
#
# Built from the repository root so the whole workspace is in scope.

ARG RUST_VERSION=1.98.1

FROM lukemathwalker/cargo-chef:latest-rust-${RUST_VERSION} AS chef
WORKDIR /build

# Reduce the workspace to a dependency manifest. Nothing here depends on our
# source, so the expensive layer below survives ordinary code changes.
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /build/recipe.json recipe.json
# Dependencies only. Cached until Cargo.toml or Cargo.lock changes — this is
# what keeps a one-line code edit from recompiling axum, tokio and sqlx.
RUN cargo chef cook --release --locked --recipe-path recipe.json

COPY . .
# query! macros verify against the committed .sqlx metadata, so the build needs
# no database.
ENV SQLX_OFFLINE=true
# Compile-time only: option_env! bakes this into the binary, so setting it at
# runtime does nothing. The smoke suite asserts the served value matches the SHA
# CI built.
ARG MEDIADIVE_GIT_SHA=dev
ENV MEDIADIVE_GIT_SHA=${MEDIADIVE_GIT_SHA}
RUN cargo build --release --locked --bin api --bin worker --bin migrate --bin smoke

# distroless: no shell, no package manager, runs as nonroot, ships CA
# certificates. Debian 13 matches the builder's glibc.
FROM gcr.io/distroless/cc-debian13 AS runtime
USER nonroot

FROM runtime AS api
COPY --from=builder /build/target/release/api /usr/local/bin/api
# The migration Job and the Rollouts analysis Job both run this same image with
# a different command, so schema, code and the test that gates promotion can
# never be deployed at different versions.
COPY --from=builder /build/target/release/migrate /usr/local/bin/migrate
COPY --from=builder /build/target/release/smoke /usr/local/bin/smoke
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/api"]

FROM runtime AS worker
COPY --from=builder /build/target/release/worker /usr/local/bin/worker
ENTRYPOINT ["/usr/local/bin/worker"]
