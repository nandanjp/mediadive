//! Configuration, read from the environment and validated at boot.
//!
//! A missing or malformed value fails the process immediately rather than
//! surfacing as a confusing runtime error later.

use std::net::SocketAddr;

pub struct Config {
    pub database_url: String,
    pub bind_addr: SocketAddr,
    pub db_max_connections: u32,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: required("DATABASE_URL")?,
            bind_addr: optional("BIND_ADDR", "0.0.0.0:8080")?.parse()?,
            db_max_connections: optional("DB_MAX_CONNECTIONS", "10")?.parse()?,
        })
    }
}

fn required(key: &str) -> anyhow::Result<String> {
    std::env::var(key).map_err(|_| anyhow::anyhow!("{key} is required"))
}

fn optional(key: &str, default: &str) -> anyhow::Result<String> {
    Ok(std::env::var(key).unwrap_or_else(|_| default.to_owned()))
}
