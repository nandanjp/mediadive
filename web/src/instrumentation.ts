/**
 * Runs once at server startup, before the first request is served.
 *
 * Importing `env` here forces validation at boot rather than lazily on the
 * first request that happens to touch it — so a misconfigured container fails
 * immediately instead of starting healthily and erroring under traffic. Mirrors
 * `Config::from_env` on the Rust side.
 */
export async function register() {
  if (process.env.NEXT_RUNTIME !== "nodejs") return;

  try {
    await import("@/env");
  } catch {
    // Next logs the validation failure but leaves the process alive and not
    // serving. Exiting turns a pod that is "running but never ready" into a
    // plain crash, which is far easier to diagnose from `kubectl get pods`.
    process.exit(1);
  }
}
