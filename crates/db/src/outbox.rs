//! Transactional outbox.
//!
//! Events carry an id and nothing else. The worker re-reads current state, so
//! stale payloads, out-of-order delivery and coalescing all stop mattering.
//!
//! Delivery is at-least-once: **every handler must be idempotent.**

use sqlx::PgExecutor;
use uuid::Uuid;

use crate::{Db, Error};

#[derive(Debug, Clone)]
pub struct Claim {
    pub id: Uuid,
    pub event_type: String,
    pub subject_id: Uuid,
    pub attempts: i32,
}

/// Append an event. Must run inside the transaction that made the change it
/// describes — that is what makes the outbox transactional.
pub async fn append<'e, E>(ex: E, event_type: &str, subject_id: Uuid) -> Result<Uuid, Error>
where
    E: PgExecutor<'e>,
{
    let id = Uuid::now_v7();
    sqlx::query!(
        "insert into platform.outbox (id, event_type, subject_id) values ($1, $2, $3)",
        id,
        event_type,
        subject_id
    )
    .execute(ex)
    .await?;
    Ok(id)
}

/// Claim up to `limit` events, skipping rows another worker holds.
///
/// A claim is a lease: rows claimed longer than `lease_secs` ago are reclaimable,
/// so a worker that dies mid-batch does not strand work.
pub async fn claim(db: &Db, limit: i64, lease_secs: f64) -> Result<Vec<Claim>, Error> {
    let claims = sqlx::query_as!(
        Claim,
        r#"
        update platform.outbox o
           set claimed_at = now(),
               attempts   = o.attempts + 1
         where o.id in (
               select id
                 from platform.outbox
                where processed_at is null
                  and (claimed_at is null
                       or claimed_at < now() - make_interval(secs => $2))
                order by created_at
                limit $1
                  for update skip locked
         )
        returning o.id, o.event_type, o.subject_id, o.attempts
        "#,
        limit,
        lease_secs
    )
    .fetch_all(db)
    .await?;
    Ok(claims)
}

pub async fn mark_processed(db: &Db, id: Uuid) -> Result<(), Error> {
    sqlx::query!(
        "update platform.outbox set processed_at = now(), last_error = null where id = $1",
        id
    )
    .execute(db)
    .await?;
    Ok(())
}

pub async fn mark_failed(db: &Db, id: Uuid, error: &str) -> Result<(), Error> {
    sqlx::query!(
        "update platform.outbox set claimed_at = null, last_error = $2 where id = $1",
        id,
        error
    )
    .execute(db)
    .await?;
    Ok(())
}
