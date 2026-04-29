//! Library crate for the Cube Practice backend. Exposes the modules so
//! integration tests can call into them directly (e.g. `cases::list_for_user`).
//! `src/main.rs` is a thin wrapper that calls `run()`.

use std::time::Duration;

use axum::Router;
use tower_http::{cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub mod auth;
pub mod cases;
pub mod config;
pub mod db;
pub mod email;
pub mod error;
pub mod routes;
pub mod srs;
pub mod state;
pub mod study;

use crate::config::Config;
use crate::state::AppState;

pub async fn run() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,cube_backend=debug")),
        )
        .with(tracing_subscriber::fmt::layer().compact())
        .init();

    let config = Config::from_env()?;
    let bind_addr = format!("0.0.0.0:{}", config.port);

    let pool = match &config.database_url {
        Some(url) => {
            tracing::info!("connecting to database");
            db::connect(url).await?
        }
        None => {
            anyhow::bail!(
                "DATABASE_URL is not set. Add it to backend/.env (see .env.example)."
            );
        }
    };

    let state = AppState::new(pool, config.clone());

    let cors = CorsLayer::new()
        .allow_origin(config.frontend_url.parse::<axum::http::HeaderValue>()?)
        .allow_credentials(true)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PATCH,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
            axum::http::header::COOKIE,
        ]);

    let app = Router::new()
        .nest("/api/v1", routes::router())
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::new(Duration::from_secs(15)))
        .layer(cors);

    tracing::info!("listening on {bind_addr}");
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
