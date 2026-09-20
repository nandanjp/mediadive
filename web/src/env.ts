import { createEnv } from "@t3-oss/env-nextjs";
import { z } from "zod";

/**
 * Validated configuration. Importing `env` anywhere guarantees the process was
 * started with usable configuration — a missing or malformed value fails the
 * build or boot rather than surfacing as `undefined` deep in a request.
 *
 * Mirrors the Rust side, where `Config::from_env` validates at boot.
 */
export const env = createEnv({
  /** Server-only. These never enter the browser bundle. */
  server: {
    API_URL: z.url(),
  },

  /**
   * Anything the browser may read must carry the `NEXT_PUBLIC_` prefix, and
   * t3-env enforces that at the type level — a server secret cannot be added
   * here by accident.
   */
  client: {},

  /**
   * Next no longer statically analyses server-side `process.env`, so only
   * client and shared variables need destructuring here.
   */
  experimental__runtimeEnv: {},

  /**
   * `next build` runs inside the Docker image with no runtime configuration, so
   * the image build sets this. Nothing else should.
   */
  skipValidation: !!process.env.SKIP_ENV_VALIDATION,

  /** An unset variable and an empty one are the same mistake. */
  emptyStringAsUndefined: true,
});
