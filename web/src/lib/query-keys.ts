/**
 * Every query key comes from here.
 *
 * Centralising them means invalidation after a mutation can target exactly the
 * right subtree — `queryKeys.media.detail(id)` invalidates one title, while
 * `queryKeys.media.all()` invalidates every media query. Keys written inline at
 * call sites drift, and a typo silently creates a second cache entry instead of
 * failing.
 */
export const queryKeys = {
  version: () => ["version"] as const,
} as const;
