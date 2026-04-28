//! Shared application state injected into every Axum handler.
//!
//! Keep this struct cheap to clone — fields are either `Arc`-internally (PgPool)
//! or short strings.

use sqlx::PgPool;

use crate::auth::password::Argon2Config;
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
    pub argon2: Argon2Config,
}

impl AppState {
    pub fn new(pool: PgPool, config: Config) -> Self {
        let argon2 = Argon2Config::from_env();
        Self {
            pool,
            config,
            argon2,
        }
    }
}
