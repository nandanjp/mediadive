"use client";

import { useQuery } from "@tanstack/react-query";

import { versionQuery } from "@/lib/queries/version";

const row = { margin: 0 } as const;

/**
 * Milestone 0's proof that the hydration path works: this renders from data the
 * server already fetched, with no loading flash and no refetch on mount.
 */
export function ApiStatus() {
  const { data, isPending, error } = useQuery(versionQuery);

  if (isPending) return <p>checking api…</p>;
  if (error) return <p>api unreachable — {error.message}</p>;

  return (
    <dl style={{ display: "grid", gridTemplateColumns: "auto 1fr", gap: "0 1rem" }}>
      <dt>api</dt>
      <dd style={row}>reachable</dd>
      <dt>revision</dt>
      <dd style={row}>{data.revision}</dd>
      <dt>version</dt>
      <dd style={row}>{data.version}</dd>
    </dl>
  );
}
