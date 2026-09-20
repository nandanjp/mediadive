import { queryOptions } from "@tanstack/react-query";
import { zVersion } from "@mediadive/api-types/zod";

import { apiFetch } from "@/lib/api";
import { queryKeys } from "@/lib/query-keys";

/**
 * One definition, shared by the server prefetch and the client `useQuery`.
 *
 * `queryOptions` binds the key to the query function's return type, so a key
 * paired with the wrong shape is a compile error rather than a cache entry
 * nobody reads.
 */
export const versionQuery = queryOptions({
  queryKey: queryKeys.version(),
  /**
   * `no-store` keeps this out of Next's data cache, which also makes any page
   * prefetching it render per request rather than at build time. Without it the
   * page is prerendered during the image build — where no API is reachable — and
   * ships with a permanent failure state baked in.
   *
   * Cache policy belongs on each query, not in `apiFetch`: public catalog pages
   * will want the opposite of this.
   */
  queryFn: () => apiFetch("/api/v1/version", zVersion, { cache: "no-store" }),
});
