//! Shared application state injected into every Axum handler.
//!
//! Keep this struct cheap to clone — fields are either `Arc`-internally
//! (PgPool, reqwest::Client) or short strings.

use sqlx::PgPool;

use crate::auth::password::Argon2Config;
use crate::auth::rate_limit::RateLimiter;
use crate::auth::recaptcha::RecaptchaVerifier;
use crate::config::Config;
use crate::email::ResendClient;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Config,
    pub argon2: Argon2Config,
    pub email: ResendClient,
    pub recaptcha: RecaptchaVerifier,
    pub rate_limit: RateLimiter,
}

impl AppState {
    pub fn new(pool: PgPool, config: Config) -> Self {
        let argon2 = Argon2Config::from_env();
        let email = ResendClient::new(config.resend_api_key.clone(), config.email_from.clone());
        let recaptcha = RecaptchaVerifier::new(
            config.recaptcha_secret_key.clone(),
            config.recaptcha_min_score,
        );
        Self {
            pool,
            config,
            argon2,
            email,
            recaptcha,
            rate_limit: RateLimiter::new(),
        }
    }
}
