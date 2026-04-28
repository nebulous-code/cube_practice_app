use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub frontend_url: String,
    pub database_url: Option<String>,
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "dev-only-jwt-secret-do-not-use-in-prod".to_string());
        if jwt_secret.len() < 32 {
            tracing::warn!("JWT_SECRET is shorter than 32 chars — fine for dev, replace in prod");
        }

        Ok(Self {
            port: env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080),
            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:5173".to_string()),
            database_url: env::var("DATABASE_URL").ok(),
            jwt_secret,
        })
    }
}
