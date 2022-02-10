pub mod server;
pub mod peer;
pub mod errors;
pub mod config;

/// Build timestamp in UTC
pub const BUILD_TIME: &str = build_time::build_time_utc!();

/// agent version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");