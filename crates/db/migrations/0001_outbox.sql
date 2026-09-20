-- The transactional outbox. Events carry ids only; the worker re-reads current
-- state, so stale payloads and out-of-order delivery cannot corrupt anything.
create schema if not exists platform;

create table platform.outbox (
    id           uuid        primary key,
    event_type   text        not null,
    subject_id   uuid        not null,
    created_at   timestamptz not null default now(),
    claimed_at   timestamptz,
    processed_at timestamptz,
    attempts     integer     not null default 0,
    last_error   text
);

-- The claim query scans only unfinished work.
create index outbox_unprocessed_idx
    on platform.outbox (created_at)
    where processed_at is null;
