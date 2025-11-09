use std::sync::OnceLock;

use anyhow::Context;

use crate::outputln;

#[derive(Debug)]
pub struct Object {
    pub log_days_retention: usize,
    pub db_username: String,
    pub db_password: String,
    pub db_sync_interval: u64,
}

static OBJECT: OnceLock<Object> = OnceLock::new();

pub fn object<'a>() -> &'a Object {
    OBJECT.get().expect("config was not initialized")
}

pub fn load() -> anyhow::Result<()> {
    let object: Object = Object {
        log_days_retention: std::env::var("LOG_RETENTION")
            .context("missing 'LOG_RETENTION'")?
            .parse()
            .context("bad 'LOG_RETENTION' value")?,
        db_username: std::fs::read_to_string(
            std::env::var("DB_USERNAME").context("missing 'DB_USERNAME'")?,
        )
        .context("failed to open 'DB_USERNAME' file")?,
        db_password: std::fs::read_to_string(
            std::env::var("DB_PASSWORD").context("missing 'DB_PASSWORD'")?,
        )
        .context("failed to open 'DB_PASSWORD' file")?,
        db_sync_interval: std::env::var("PINGER_DB_SYNC_INTERVAL")
            .context("missing 'PINGER_DB_SYNC_INTERVAL'")?
            .parse()
            .context("bad 'PINGER_DB_SYNC_INTERVAL' value")?,
    };

    OBJECT.set(object).expect("config was already initialized");

    outputln!("configuration was loaded");

    Ok(())
}
