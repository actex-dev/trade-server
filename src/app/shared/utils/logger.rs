use tracing_subscriber::{fmt, EnvFilter};
use tracing_subscriber::prelude::*;

pub fn init() {
    // Default to INFO if RUST_LOG not set
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // Human-readable formatter with timestamps
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_target(true).with_thread_ids(false).with_file(false))
        .init();
}