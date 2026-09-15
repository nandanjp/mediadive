# Domain Context

The vocabulary of the system. Every term used in code, schemas, and other docs is
defined here exactly once.

## Entities

| Entity | Definition |
|---|---|
| **Account** | Authentication identity. Holds email, password credential and/or linked Google identity, role (`member`/`admin`), and blocked state. |
| **Profile** | The public face of an Account: username slug, display name, avatar, bio, privacy switch. |
| **Media** | One watchable work at **season** granularity. Flavour is `anime`, `drama`, or `movie`. Created only by import. |
| **Genre** | Controlled vocabulary. Finite, curated, filterable. |
| **Tag** | Free-form descriptor. High cardinality, filterable, not curated. |
| **Studio** | Production company or broadcast network. |
| **Person** | A real human. Global — exists independently of any Media. |
| **Character** | A fictional role. Belongs to exactly one Media. |
| **Credit** | Links a Person to a Media with a role type. References a Character when the role is performed; a Credit without one is crew. |
| **MediaRelation** | Typed edge between two Media — sequel, prequel, other season, spin-off, adaptation. |
| **Library** | One per Member. The set of Media that Member tracks. |
| **LibraryEntry** | One Media in one Library. Carries status, progress, and that Member's rating. |
| **List** | A named, curated collection of Media authored by a Member or Admin. Carries a descriptive label and a public/private setting. Admin lists may be featured. |
| **Review** | Optional long-form prose about one Media by one Member. Rich-text document supporting inline spoiler marks. Public or private. |
| **Article** | Admin-authored editorial content: cover image, rich-text body, draft/published state, structural links to Media. |
| **Report** | A Member-filed flag against a Review or an Account. |
| **ModerationAction** | An Admin action taken against a Review or an Account, with reason. |
| **TitleRequest** | A Member's request for a Media absent from the catalog. Resolved by an Admin into an import. |

## Derived values

| Value | Definition |
|---|---|
| **Rating aggregate** | Per Media. Computed from LibraryEntry ratings. |
| **Analytics** | Per Member. Counts by genre, counts by country, estimated total watch time. |

## Invariants

1. A Media must be in a Member's Library before that Member can rate or review it.
2. Ratings live on **LibraryEntry**, never on Review. A Review is optional prose, and its privacy setting governs prose only.
3. Rating aggregates **include** private Members' ratings and **exclude** blocked Accounts' ratings. Privacy is a choice; blocking is a sanction.
4. At most one LibraryEntry and at most one Review per (Member, Media).
5. A List's label is descriptive. Membership is never validated against it.
6. Profile privacy is a master switch: profile, library, lists, and reviews are public together or hidden together. Per-object privacy may only further restrict, never loosen.
7. Media is created and updated only by import. Re-import presents a field-level diff that an Admin applies; there is no automatic overwrite.
8. A Character belongs to exactly one Media. A Person does not.
9. Seasons of one show are separate Media, disambiguated in the title, joined by MediaRelation, and surfaced individually in search.
10. A blocked Account cannot sign in, and its content is hidden.
11. Watch time is an estimate. Per-flavour default durations apply when a source omits them.

## Terminology

**Catalog** is the browse-and-search surface over Media. It is not an entity; its
filters are Media attributes.

**Flavour** is the kind of work a Media is. The set is closed for v1 and the model
is deliberately flavour-agnostic so the set can grow.

**Member** is an Account with role `member`; **Admin** is an Account with role
`admin`; **Visitor** is an unauthenticated request.
