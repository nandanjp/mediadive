# Functional Requirements

Terms are defined in [CONTEXT.md](./CONTEXT.md). Rules stated there are not repeated here.

## Roles

Cumulative. **Visitor** → **Member** → **Admin**.

## Capabilities

`•` = permitted. Flagged capabilities ship dark and are enabled per user.

| Capability | Visitor | Member | Admin | Status |
|---|:--:|:--:|:--:|---|
| Browse and filter the catalog | • | • | • | v1 |
| Search | • | • | • | v1 |
| View Media, Person, and Character pages | • | • | • | v1 |
| View public Profiles, Libraries, and Lists | • | • | • | v1 |
| Read public Reviews and published Articles | • | • | • | v1 |
| Register and sign in | • | — | — | v1 |
| Add, update, and remove LibraryEntries | | • | • | v1 |
| Rate a Media | | • | • | v1 |
| Write, edit, and delete Reviews (public or private) | | • | • | v1 |
| Create, edit, and delete Lists (public or private) | | • | • | v1 |
| Edit Profile and set its privacy | | • | • | v1 |
| View own Analytics | | • | • | v1 |
| Submit a TitleRequest | | • | • | v1 |
| File a Report | | • | • | v1 |
| Follow other Members | | • | • | flagged |
| Vote on Reviews and Lists | | • | • | flagged |
| Comment | | • | • | flagged |
| Import Media, and re-import via diff | | | • | v1 |
| Edit catalog records | | | • | v1 |
| Resolve TitleRequests | | | • | v1 |
| Author Articles and link them to Media | | | • | v1 |
| Publish featured Lists | | | • | v1 |
| Act on Reports: hide a Review, block an Account | | | • | v1 |

## Identity

Registration and sign-in by **Google OAuth** or **email and password**. Blocking
acts on the Account: sign-in is refused and all authored content is hidden.

## Search

Matches against Media title, Person name, List name, and Article title.
**Native-language title matching (Korean, Japanese, Chinese script) is a
first-class requirement**, not a later refinement.

Filters: genre, tag, country, episode count, flavour.

Each season is its own result.

## Catalog ingestion

Sources are **AniList** for anime and **TMDB** for drama and movie. Field mapping
is deferred to implementation.

Two entry points, both Admin-driven:

1. **Seed** — an initial set of ~100 curated titles, loaded by script.
2. **Admin import** — an Admin searches an upstream source, selects a result, and
   imports it. Re-importing an existing Media presents a field-level diff the
   Admin applies selectively.

There is no user-triggered ingestion. **TitleRequest** is the channel by which
Members signal that something is missing.

## Moderation

Members file Reports against Reviews or Accounts. Admins act on them by hiding a
Review or blocking an Account.

## Analytics

Per Member, derived on read: counts by genre, counts by country, and estimated
total watch time.

## Feature flags

A single engine gates both backend and frontend. Evaluation is **per user**, with
account id, role, and cohort in the evaluation context. Application behaviour must
remain coherent with any flag off.

Code targets the **OpenFeature** SDK specification; the provider is chosen during
architecture design.

## Out of scope for v1

Airing calendar, and therefore Season and Episode as entities · AI and
recommendation systems · notifications · messaging · forums · community editing of
the catalog · flavours beyond anime, drama, and movie.
