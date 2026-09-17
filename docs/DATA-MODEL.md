# Data Model

Entity-relationship level. **Migrations are the source of truth for column types**
— this document records structure and the rules DDL cannot express.

Terms: [CONTEXT.md](./CONTEXT.md) · Flows: [FLOWS.md](./FLOWS.md)

## Conventions

- **UUIDv7** primary keys everywhere — time-ordered for index locality, no count
  leakage through sequential public URLs.
- `created_at` / `updated_at` as `timestamptz` on every mutable table.
- **Enums are text with a `CHECK` constraint**, mapped to Rust enums by sqlx.
  Postgres `ENUM` types cannot drop a value, which fits expand/contract badly.
- **Foreign keys are real and cross schemas.** The schema split marks ownership;
  it does not forbid referential integrity.
- **Hard delete for user actions; flags for moderation.** Soft-deleting everything
  means one forgotten `WHERE` leaks removed data.
- **Plain views for join readability. No materialized views in v1** — refresh
  granularity is wrong for per-entity invalidation, and Redis already caches the
  composite reads.

---

## `identity`

| Table | Key columns | Notes |
|---|---|---|
| `account` | `email` citext unique · `password_hash?` · `google_sub?` unique · `role` CHECK(member, admin) · `email_verified_at?` · `blocked_at?` | Both credentials nullable: Google-only accounts have no hash. |
| `profile` | `account_id` PK/FK · `username` citext unique · `display_name` · `avatar_key?` · `bio?` · `is_private` | 1:1 with account. `is_private` is the master switch. |
| `session` | `account_id` FK · `token_hash` unique · `expires_at` · `last_seen_at` | Cookie carries the opaque token; only its hash is stored. `last_seen_at` drives sliding renewal. |
| `auth_token` | `account_id` FK · `token_hash` unique · `purpose` CHECK(verify_email, reset_password) · `expires_at` · `used_at?` | One table for both; single-use enforced by `used_at`. |

---

## `catalog`

| Table | Key columns | Notes |
|---|---|---|
| `media` | `flavour` CHECK(anime, drama, movie) · `source` CHECK(anilist, tmdb) · `external_id` · **unique(source, external_id)** · `title` · `original_title` · `synopsis` · `year` · `country` · `language` · `episode_count` · `duration_minutes` · `studio_id?` · `poster_thumb_key?` · `poster_full_key?` | One row per season. Title disambiguates. |
| `genre` | `name` · `slug` unique | Created on first sight from upstream. |
| `genre_alias` | `source` · `upstream_value` · `genre_id` · **unique(source, upstream_value)** | Survives a merge so later imports resolve without re-deciding. |
| `media_genre` | PK(`media_id`, `genre_id`) | |
| `tag` / `media_tag` | same shape as genre | No alias table — tags are never merged. |
| `studio` | `name` · `source?` · `external_id?` unique with source | |
| `person` | `name` · `native_name?` · `source` · `external_id` · **unique(source, external_id)** · `birthdate?` · `biography?` · `image_key?` | Deduplicated by upstream id only. |
| `character` | `media_id` FK · `name` · `native_name?` | Never deduplicated — belongs to one media. |
| `credit` | `media_id` FK · `person_id` FK · `character_id?` FK · `role_type` CHECK(lead, supporting, guest, cameo, crew) · `department?` · `billing_order` | Null character ⇒ crew. |
| `media_relation` | `from_media_id` · `to_media_id` · `relation_type` CHECK(sequel, prequel, other_season, spin_off, adaptation) | **Both directions stored** with inverse types, so reads never need a `UNION`. |
| `import_draft` | `source` · `external_id` · `flavour` · `payload` jsonb · `media_id?` · `created_by` · `status` CHECK(open, committed, discarded) · `expires_at` | `media_id` set ⇒ re-import, and the UI diffs against it. |

---

## `library`

| Table | Key columns | Notes |
|---|---|---|
| `library_entry` | `account_id` · `media_id` · **unique(account_id, media_id)** · `status` CHECK(watching, completed, plan_to_watch, dropped) · `episodes_watched` · `rating?` CHECK(1–10) · `started_at?` · `completed_at?` | The rating lives here, not on Review. No `position` — Library sorts are derived. |
| `media_rating` | `media_id` PK · `avg_rating` numeric(4,2) · `rating_count` · `updated_at` | Recomputed in the same transaction as a rating write, excluding blocked accounts. |

---

## `review`

| Table | Key columns | Notes |
|---|---|---|
| `review` | `account_id` · `media_id` · **unique(account_id, media_id)** · `body` jsonb · `body_text` · `is_public` · `hidden_at?` | `body` is ProseMirror JSON, validated against a node/mark allowlist on write. `body_text` is flattened for search and excerpts. `hidden_at` is moderation; it never affects `media_rating`. |

---

## `list`

| Table | Key columns | Notes |
|---|---|---|
| `list` | `account_id` · `name` · `label?` · `description?` · `is_public` · `is_featured` | Label is descriptive and never validated against membership. `is_featured` marks admin-published lists. |
| `list_item` | `list_id` · `media_id` · **unique(list_id, media_id)** · `position` · `note?` | Hand-arranged. Position is **not** unique-constrained — swaps would need deferred constraints; reordering renumbers the list in one statement. |

---

## `article`

| Table | Key columns | Notes |
|---|---|---|
| `article` | `author_account_id` · `title` · `slug` unique · `cover_image_key?` · `body` jsonb · `body_text` · `status` CHECK(draft, published) · `published_at?` | Same rich-text rules as Review. |
| `article_media` | PK(`article_id`, `media_id`) | Powers "mentioned in these articles" on the media page. |

---

## `moderation`

Report and ModerationAction each target **either** a Review **or** an Account.
Postgres cannot foreign-key a polymorphic pair, so both use an **exclusive arc**:
two nullable FK columns with a `CHECK` that exactly one is set. Real integrity, at
the cost of nullable columns.

| Table | Key columns | Notes |
|---|---|---|
| `report` | `reporter_account_id` · `target_review_id?` · `target_account_id?` · `reason` CHECK · `note?` · `status` CHECK(open, actioned, dismissed) | **Partial unique** on (reporter, target) `WHERE status = 'open'` — one open report per reporter per target. |
| `moderation_action` | `admin_account_id` · `target_review_id?` · `target_account_id?` · `action` CHECK(hide_review, unhide_review, block_account, unblock_account) · `reason` | Append-only audit trail. |

---

## `platform`

| Table | Key columns | Notes |
|---|---|---|
| `outbox` | `event_type` · `subject_id` · `created_at` · `claimed_at?` · `processed_at?` · `attempts` · `last_error?` | **No payload column.** Events carry an id; the worker re-reads current state, so stale payloads, out-of-order delivery and coalescing all stop mattering. |
| `title_request` | `account_id` · `query_text` · `flavour_hint?` · `status` CHECK(open, imported, rejected) · `resolved_media_id?` | |

### Events

| Event | Subject | Worker does |
|---|---|---|
| `media.changed` | media | Reindex; fetch cover art if the source URL changed |
| `media.deleted` | media | Remove document and stored images |
| `rating.changed` | media | Patch the search document (coalesced by subject) |
| `account.blocked` / `unblocked` | account | Recompute every aggregate the account contributed to; add or remove its list documents |
| `profile.visibility.changed` | account | Add or remove that account's list documents |
| `list.changed` | list | Upsert or remove the list document |
| `article.published` / `unpublished` | article | Upsert or remove the article document |

Reviews are not indexed — search covers media, person, list and article only.

---

## Indexes worth stating

- `library_entry(media_id)` — aggregate recomputation.
- `library_entry(account_id, status)` — library views and analytics.
- `credit(person_id)` and `credit(media_id)` — both directions of the cast join.
- `outbox` partial index `WHERE processed_at IS NULL` — the claim query scans only
  unfinished work.
- `report` partial unique `WHERE status = 'open'`.

## Deferred

| Change | Trigger |
|---|---|
| `rating_sum` / `rating_count` columns replacing full recomputation | Aggregate recomputation appears in query timings |
| Purge worker for hidden reviews and blocked accounts; account deletion | Retention or privacy requirement appears |
| Materialized views for trending and analytics rollups | Those queries stop being fast live |
| Person merge tool | Cross-source duplicate Persons become visible enough to matter |
