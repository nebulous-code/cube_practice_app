mod config;
mod db;
mod error;
mod routes;

use std::time::Duration;

use axum::Router;
use tower_http::{
    cors::CorsLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info,oll_backend=debug")))
        .with(tracing_subscriber::fmt::layer().compact())
        .init();

    let config = Config::from_env()?;
    let bind_addr = format!("0.0.0.0:{}", config.port);

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
        .allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::ACCEPT]);

    let app = Router::new()
        .nest("/api/v1", routes::router())
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::new(Duration::from_secs(15)))
        .layer(cors);

    tracing::info!("listening on {bind_addr}");
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
