//! Outbox consumer.
//!
//! Delivery is at-least-once, so **every handler must be idempotent**. No
//! handlers exist yet — milestone 0 proves the claim loop, not the work.

use std::time::Duration;

use mediadive_db::outbox;
use tracing_subscriber::EnvFilter;

const BATCH: i64 = 32;
const LEASE_SECS: f64 = 300.0;
const IDLE: Duration = Duration::from_secs(2);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let url =
        std::env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?;
    let db = mediadive_db::connect(&url, 5).await?;
    tracing::info!("worker started");

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => break,
            result = drain(&db) => {
                if let Err(error) = result {
                    tracing::error!(%error, "drain failed");
                }
                tokio::time::sleep(IDLE).await;
            }
        }
    }

    tracing::info!("worker stopped");
    Ok(())
}

async fn drain(db: &mediadive_db::Db) -> anyhow::Result<()> {
    for claim in outbox::claim(db, BATCH, LEASE_SECS).await? {
        // Unroutable events are completed rather than retried forever; the log
        // line is the signal that a handler is missing.
        tracing::warn!(
            event_type = %claim.event_type,
            subject_id = %claim.subject_id,
            attempts = claim.attempts,
            "no handler registered"
        );
        outbox::mark_processed(db, claim.id).await?;
    }
    Ok(())
}
