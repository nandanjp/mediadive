//! Run pending migrations and exit. Invoked by the Helm `pre-upgrade` hook.
//!
//! Migrations must be backward-compatible — both colours of a blue-green
//! rollout run against the schema this produces. See `docs/ARCHITECTURE.md`.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url =
        std::env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL is required"))?;
    let db = mediadive_db::connect(&url, 1).await?;
    mediadive_db::migrate(&db).await?;
    println!("migrations applied");
    Ok(())
}
