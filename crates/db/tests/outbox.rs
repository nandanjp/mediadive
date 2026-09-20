//! Outbox claim semantics: lease, idempotent completion, no double delivery.

use mediadive_db::outbox;
use uuid::Uuid;

#[sqlx::test]
async fn claim_leases_then_completes(pool: sqlx::PgPool) -> sqlx::Result<()> {
    let subject = Uuid::now_v7();
    outbox::append(&pool, "media.changed", subject)
        .await
        .expect("append");

    let claimed = outbox::claim(&pool, 10, 300.0).await.expect("first claim");
    assert_eq!(claimed.len(), 1);
    assert_eq!(claimed[0].event_type, "media.changed");
    assert_eq!(claimed[0].subject_id, subject);
    assert_eq!(claimed[0].attempts, 1, "claiming increments attempts");

    // Still leased, so a second worker sees nothing.
    let again = outbox::claim(&pool, 10, 300.0).await.expect("second claim");
    assert!(again.is_empty(), "a leased row must not be claimed twice");

    outbox::mark_processed(&pool, claimed[0].id)
        .await
        .expect("mark processed");

    let after = outbox::claim(&pool, 10, 300.0).await.expect("third claim");
    assert!(after.is_empty(), "a processed row is never reclaimed");
    Ok(())
}

#[sqlx::test]
async fn expired_lease_is_reclaimable(pool: sqlx::PgPool) -> sqlx::Result<()> {
    outbox::append(&pool, "media.changed", Uuid::now_v7())
        .await
        .expect("append");

    outbox::claim(&pool, 10, 300.0).await.expect("claim");

    // A zero-second lease means the previous claim has already expired — the
    // case of a worker that died mid-batch.
    let reclaimed = outbox::claim(&pool, 10, 0.0).await.expect("reclaim");
    assert_eq!(reclaimed.len(), 1, "a dead worker must not strand work");
    assert_eq!(reclaimed[0].attempts, 2);
    Ok(())
}

#[sqlx::test]
async fn failure_releases_the_lease(pool: sqlx::PgPool) -> sqlx::Result<()> {
    outbox::append(&pool, "media.changed", Uuid::now_v7())
        .await
        .expect("append");
    let claimed = outbox::claim(&pool, 10, 300.0).await.expect("claim");

    outbox::mark_failed(&pool, claimed[0].id, "upstream timeout")
        .await
        .expect("mark failed");

    let retried = outbox::claim(&pool, 10, 300.0).await.expect("retry claim");
    assert_eq!(retried.len(), 1, "a failed row is retried immediately");
    Ok(())
}
