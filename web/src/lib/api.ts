import type { z } from "zod";
import { zProblem } from "@mediadive/api-types/zod";

import { env } from "@/env";

/**
 * Server-side requests go straight to the API over its own URL. Browser
 * requests use a relative path, which `next.config.ts` rewrites to the same
 * destination — so they stay same-origin and carry the session cookie.
 *
 * This is why a query function can be shared between a server prefetch and a
 * client `useQuery` without either knowing where it runs.
 */
function baseUrl(): string {
  return typeof window === "undefined" ? env.API_URL : "";
}

/**
 * An error response from the API, parsed into the shape `docs/API.md` defines.
 *
 * `code` is stable and safe to branch on; `detail` is prose and may change
 * between releases.
 */
export class ApiError extends Error {
  constructor(
    readonly status: number,
    readonly code: string,
    detail: string,
    readonly fieldErrors?: Record<string, string[]>,
  ) {
    super(detail);
    this.name = "ApiError";
  }
}

/**
 * Fetch and validate.
 *
 * The generated zod schema is the runtime half of the contract: a response that
 * does not match fails here, at the boundary, rather than surfacing later as an
 * undefined field in a component. The schema is generated from the same
 * `openapi.json` as the types, so the two cannot disagree.
 *
 * `init` is passed through untouched so callers keep full control of Next's
 * caching and revalidation per request.
 */
export async function apiFetch<TSchema extends z.ZodType>(
  path: string,
  schema: TSchema,
  init?: RequestInit,
): Promise<z.infer<TSchema>> {
  const response = await fetch(`${baseUrl()}${path}`, init);

  if (!response.ok) {
    throw await toApiError(response);
  }

  return schema.parse(await response.json());
}

/**
 * Error bodies are best-effort: a failure from a proxy or ingress will not be
 * problem+json, so a body that does not parse still yields a usable error.
 */
async function toApiError(response: Response): Promise<ApiError> {
  try {
    const problem = zProblem.parse(await response.json());
    return new ApiError(
      problem.status,
      problem.code,
      problem.detail,
      problem.errors ?? undefined,
    );
  } catch {
    return new ApiError(
      response.status,
      "unexpected_response",
      `${response.status} ${response.statusText}`,
    );
  }
}
