# API Conventions

Contracts are generated, not hand-written — see
[ADR-0007](./adr/0007-generated-contracts.md). These are the rules the generator
encodes.

## Basics

- Base path `/api/v1`. One first-party client makes versioning near-free
  insurance rather than necessary ceremony.
- JSON is **`snake_case`** — serde's default, matching column names, with the
  generated TypeScript client carrying it through.
- Timestamps are RFC 3339, UTC.
- Plural resource nouns, nested no more than one level.
- Authentication is the session cookie. There is no `Authorization` header.
  State-changing requests carry `X-CSRF-Token`.
- **No idempotency keys.** Nothing here is a payment, and the one write worth
  protecting — import commit — is guarded by `unique(source, external_id)`.

## Errors

RFC 9457 problem details, plus a `code` clients can branch on without
string-matching prose, plus an `errors` map for field-level validation.

```json
{
  "type": "about:blank",
  "title": "Unprocessable Entity",
  "status": 422,
  "detail": "Registration could not be completed.",
  "code": "email_already_registered",
  "errors": { "email": ["already in use"] }
}
```

`code` is stable and machine-readable; `detail` is prose and may change.

## Pagination

**Cursor by default.** Offset pagination duplicates rows when something is
inserted mid-scroll and skips rows when something is deleted — the faster content
arrives, the worse it gets — and deep offsets make Postgres discard every skipped
row before returning anything.

```json
{ "items": [ … ], "next_cursor": "eyJpZCI6IjAxOTI4…" }
```

The cursor is opaque base64 encoding the sort key of the last row. Because UUIDv7
is time-ordered, **for anything sorted by creation time the id alone is the
cursor**; other sorts encode `(sort_key, id)`, the id breaking ties so a shared
timestamp cannot skip or repeat a row.

**Offset for search only.** Relevance is not a stable stored key you can seek
into, Meilisearch is natively offset-based, and a result total — which cursors
cannot cheaply provide — is exactly what a search page wants.

```json
{ "items": [ … ], "total": 1240, "offset": 0, "limit": 20 }
```

**List items are not paginated.** Lists are short and positionally ordered.

| Surface | Order | Pagination |
|---|---|---|
| Reviews on a media page | newest first | cursor |
| Member library | sort chosen at view time | cursor on `(sort_key, id)` |
| Articles | newest published | cursor |
| Admin report queue | oldest open first | cursor |
| Search | relevance | offset |
| List items | stored position | none |
