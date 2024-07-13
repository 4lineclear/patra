//! The core server implementations
#![allow(clippy::enum_glob_use)]

use std::{sync::Arc, time::Duration};

use axum::Router;
use libreauth::pass::{Error as HashError, HashBuilder};
use persist::error::ConnectionError;
use state::{Context, ValidationRules};
use thiserror::{self, Error};
use tower_http::{
    catch_panic::CatchPanicLayer, compression::CompressionLayer, timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{
    filter::FromEnvError, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

// Re-Exports for binary crates
pub use anyhow;
pub use axum;
pub use tokio;
pub use tracing;
pub use tracing_subscriber;

/// Handles persist
pub mod persist;
/// Handles state
pub mod state;

/// Creates the standard router
///
/// # Errors
///
/// See [`CreateRouterError`]
pub async fn router(url: String) -> Result<Router, CreateRouterError> {
    let api = Api {
        ctx: Context::new(
            url,
            HashBuilder::new().finalize()?,
            ValidationRules {
                pass_min: 8,
                pass_max: 128,
                name_min: 1,
                name_max: 128,
            },
        )
        .await?
        .into(),
    };

    Ok(Router::new()
        .layer((
            CompressionLayer::new(),
            TraceLayer::new_for_http(),
            TimeoutLayer::new(Duration::from_secs(4)),
            CatchPanicLayer::new(),
        ))
        .with_state(api))
}

/// an error while creating the router
#[derive(Debug, Error)]
pub enum CreateRouterError {
    /// db connection/setup error
    #[error(transparent)]
    ConnectionError(#[from] ConnectionError),
    /// db connection/setup error
    #[error(transparent)]
    HashError(#[from] HashError),
}

/// The state of the router
#[derive(Debug, Clone)]
pub struct Api {
    #[allow(dead_code)]
    ctx: Arc<Context>,
}

/// Creates a logging object
///
/// # Errors
///
/// Fails when the filter is not installed
///
/// # Panics
///
/// Never
pub fn create_logging() -> Result<impl SubscriberInitExt + SubscriberExt, FromEnvError> {
    let fitler_layer = EnvFilter::builder()
        .with_default_directive(LevelFilter::OFF.into())
        .from_env()?
        .add_directive("axum=trace".parse().unwrap())
        .add_directive("core_server=trace".parse().unwrap())
        .add_directive("dev_server=trace".parse().unwrap())
        .add_directive("tower_http=trace".parse().unwrap());

    Ok(tracing_subscriber::registry()
        .with(fitler_layer)
        .with(tracing_subscriber::fmt::layer()))
}
