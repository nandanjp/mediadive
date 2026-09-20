//! Connection pooling, migrations, and the outbox.

pub mod outbox;

use sqlx::migrate::Migrator;
use sqlx::postgres::PgPoolOptions;

pub type Db = sqlx::PgPool;

pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("migration: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
}

pub async fn connect(url: &str, max_connections: u32) -> Result<Db, Error> {
    Ok(PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(url)
        .await?)
}

/// Readiness probe. Keeps SQL out of the api crate.
pub async fn ping(db: &Db) -> Result<(), Error> {
    sqlx::query!("select 1 as one").fetch_one(db).await?;
    Ok(())
}

/// Applied by the Helm `pre-upgrade` hook, never on service boot.
pub async fn migrate(db: &Db) -> Result<(), Error> {
    MIGRATOR.run(db).await?;
    Ok(())
}
