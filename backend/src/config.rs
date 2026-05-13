use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub frontend_url: String,
    pub database_url: Option<String>,
    pub jwt_secret: String,
    pub resend_api_key: String,
    pub email_from: String,
    pub turnstile_secret_key: String,
    pub account_deletion_hmac_secret: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "dev-only-jwt-secret-do-not-use-in-prod".to_string());
        if jwt_secret.len() < 32 {
            tracing::warn!("JWT_SECRET is shorter than 32 chars — fine for dev, replace in prod");
        }

        // HMAC secret for hashing deleted-account emails. Must be stable forever:
        // rotating it makes existing audit rows unverifiable. Plaintext fallback
        // is fine for dev; prod must set a real value.
        let account_deletion_hmac_secret = env::var("ACCOUNT_DELETION_HMAC_SECRET")
            .unwrap_or_else(|_| "dev-only-account-deletion-hmac-secret".to_string());
        if account_deletion_hmac_secret.len() < 32 {
            tracing::warn!(
                "ACCOUNT_DELETION_HMAC_SECRET is shorter than 32 chars — fine for dev, replace in prod"
            );
        }

        Ok(Self {
            port: env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080),
            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:5173".to_string()),
            database_url: env::var("DATABASE_URL").ok(),
            jwt_secret,
            resend_api_key: env::var("RESEND_API_KEY").unwrap_or_default(),
            email_from: env::var("EMAIL_FROM")
                .unwrap_or_else(|_| "Quiet Cube <onboarding@resend.dev>".to_string()),
            turnstile_secret_key: env::var("TURNSTILE_SECRET_KEY").unwrap_or_default(),
            account_deletion_hmac_secret,
        })
    }
}
