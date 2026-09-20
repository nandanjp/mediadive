import { HydrationBoundary, dehydrate } from "@tanstack/react-query";

import { getQueryClient } from "@/lib/query-client";
import { versionQuery } from "@/lib/queries/version";

import { ApiStatus } from "./api-status";

/**
 * The pattern every interactive page follows: fetch on the server, pass the
 * dehydrated cache down, and let client components read it without a second
 * request.
 *
 * `prefetchQuery` does not throw, so an unreachable API renders the page and
 * lets the client component report the failure — rather than returning a 500.
 */
export default async function Page() {
  const queryClient = getQueryClient();
  await queryClient.prefetchQuery(versionQuery);

  return (
    <main
      style={{
        fontFamily: "ui-monospace, monospace",
        padding: "3rem",
        lineHeight: 1.6,
      }}
    >
      <h1 style={{ fontSize: "1rem", fontWeight: 600, margin: 0 }}>mediadive</h1>
      <HydrationBoundary state={dehydrate(queryClient)}>
        <ApiStatus />
      </HydrationBoundary>
    </main>
  );
}
