use clap::Parser;
use maoer_agent::server::cli;
use maoer_agent::server::cli::Args;
use tracing;
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()
    .add_directive("agent=debug".parse()?))
    .with_span_events(FmtSpan::FULL).init();

    let args = Args::parse();
    tracing::info!("loaded : 
    {}", args);

    cli::run(&args)
}