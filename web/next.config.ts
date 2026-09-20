import type { NextConfig } from "next";

const config: NextConfig = {
  // Self-contained server bundle — the Docker image copies .next/standalone
  // and needs no node_modules.
  output: "standalone",

  // The generated contract ships as TypeScript source, not a build artifact.
  transpilePackages: ["@mediadive/api-types"],

  /**
   * The browser always calls the API at a relative `/api/...` path, so requests
   * stay same-origin and carry the session cookie.
   *
   * In production Traefik routes `/api/*` to the API and everything else to web
   * under one hostname, so nothing is needed here. Locally there is no ingress,
   * so Next stands in for it — enabled explicitly by `DEV_API_PROXY`.
   *
   * `process.env` rather than the validated `env`, and guarded rather than
   * unconditional, because Next evaluates rewrites at **build time** and bakes
   * the destination into the route manifest. The image build has no runtime
   * configuration, so an unconditional rewrite would bake in `undefined`.
   */
  async rewrites() {
    if (!process.env.DEV_API_PROXY) return [];
    return [
      {
        source: "/api/:path*",
        destination: `${process.env.API_URL}/api/:path*`,
      },
    ];
  },
};

export default config;
