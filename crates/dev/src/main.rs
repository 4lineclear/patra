//! Main entrypoint
#![allow(clippy::wildcard_imports)]

use std::env;

use core_server::*;

use anyhow::{Context, Result};
use sqlx::PgPool;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::util::SubscriberInitExt;

/// Handles input signal protocols
pub mod signal;

#[tokio::main]
async fn main() -> Result<()> {
    create_logging().context("Failed to read log env")?.init();
    serve().await?;
    Ok(())
}

/// Serves the given router & inits the given subscriber
///
/// # Errors
///
/// Fails when either [`TcpListener`] or [`axum::serve()`] does
pub async fn serve() -> Result<()> {
    let pool = PgPool::connect(env!("DATABASE_URL")).await?;
    let router = router(pool).await.context("Failed to create router")?;
    let listener = TcpListener::bind(concat!("127.0.0.1:", env!("SERVER_PORT"))).await?;
    let address = listener.local_addr()?;

    info!("Server Opened, listening on {address}");
    axum::serve(listener, router.router)
        .with_graceful_shutdown(signal::signal())
        .await
        .context("Axum Server Error")?;
    info!("Server Closed");

    Ok(())
}
