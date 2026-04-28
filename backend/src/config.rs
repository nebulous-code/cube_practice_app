use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub frontend_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            port: env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080),
            frontend_url: env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string()),
        })
    }
}
