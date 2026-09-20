import { defineConfig } from "@hey-api/openapi-ts";

/**
 * One generator, one source of truth. The Rust API emits `openapi.json`; this
 * produces the TypeScript types and the matching zod schemas. CI fails if
 * either output drifts from what is committed.
 *
 * Types and schemas only — no SDK, no vendored client. Requests go through a
 * thin helper in `web/src/lib/api.ts` so validation is explicit at the call
 * site and Next's per-request caching stays under our control.
 *
 * See docs/adr/0007-generated-contracts.md.
 */
export default defineConfig({
  input: "openapi.json",
  output: { path: "src", postProcess: [] },
  plugins: ["@hey-api/typescript", "zod"],
});
