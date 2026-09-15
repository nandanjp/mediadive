# ADR-0008 — GitOps deployment with blue-green rollouts

**Status:** Accepted · 2026-09-15

## Context
Deployment target is a homelab k3s cluster behind a Cloudflare tunnel. CI/CD was
a first-order requirement — changes should reach the cluster on commit — and
blue-green was requested explicitly.

## Decision
Argo CD reconciles a Helm umbrella chart from this repository. CI builds images,
pushes to GHCR, and bot-commits the new tag into `deploy/charts`. Argo Rollouts
performs blue-green with a preview service. Secrets are SOPS+age encrypted
in-repo. There is one environment; no staging.

## Consequences
- **Every migration must be backward-compatible.** Both colours run against one
  database during a cutover, so schema changes follow expand/contract and run as
  a Job before promotion. This is the deploy model reaching back into how code is
  written, and it is the most significant consequence here.
- With no staging environment, the **preview service is the verification
  surface**, and promotion is gated on smoke tests against it rather than
  automatic.
- The deployed version is visible in git history.
- The bot commit must be guarded against retriggering the pipeline.
- The worker is recreated rather than blue-greened; safety comes from idempotent
  handlers (ADR-0004), not from two concurrent generations.
