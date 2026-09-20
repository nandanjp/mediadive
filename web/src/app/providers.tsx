"use client";

import { QueryClientProvider } from "@tanstack/react-query";
import { ReactQueryDevtools } from "@tanstack/react-query-devtools";

import { getQueryClient } from "@/lib/query-client";

/**
 * `getQueryClient` is called during render rather than in a `useState`
 * initialiser because it already handles both environments: a new client per
 * server render, the same client for the life of the browser session.
 */
export function Providers({ children }: { children: React.ReactNode }) {
  const queryClient = getQueryClient();

  return (
    <QueryClientProvider client={queryClient}>
      {children}
      {/* Renders nothing in production builds. */}
      <ReactQueryDevtools initialIsOpen={false} />
    </QueryClientProvider>
  );
}
