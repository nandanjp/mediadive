import { QueryClient, isServer } from "@tanstack/react-query";

function makeQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: {
        /**
         * Hydrated data would otherwise count as stale the instant it arrives
         * and refetch on mount, discarding the work the server already did.
         */
        staleTime: 60_000,
        retry: 1,
        refetchOnWindowFocus: false,
      },
    },
  });
}

let browserQueryClient: QueryClient | undefined;

/**
 * A fresh client per server request, a singleton in the browser.
 *
 * Sharing one client across server requests would leak one user's cached data
 * into another user's response — the cache is not request-scoped on its own.
 */
export function getQueryClient(): QueryClient {
  if (isServer) return makeQueryClient();
  browserQueryClient ??= makeQueryClient();
  return browserQueryClient;
}
