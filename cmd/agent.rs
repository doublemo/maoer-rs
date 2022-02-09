use clap::Parser;
use maoer_agent::server::cli;
use maoer_agent::server::cli::Args;
use tracing::{self, event, span, Level};
use tracing_subscriber::{fmt::format::FmtSpan, EnvFilter};
use maoer_cores::make_errors;
use maoer_cores::error::{paser_errcode};
use maoer_protocols::kits::AGENT;
make_errors! (
    ErrorKind, 
    AGENT,
    Miter => "miter",
    Round => "round",
    Bevel => "bevel",
    Inherit => "inherit",
);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()
    .add_directive("agent=debug".parse()?))
    .with_span_events(FmtSpan::FULL).init();

    let s = ErrorKind::Inherit;
    let e = s.as_error();
    println!("dddd-d-- {} {}", e, s.enum_index());
    println!("code----> {} {:?}", e.code, paser_errcode(e.code));
    let args = Args::parse();
    tracing::info!("loaded : 
    {}", args);

    let span = span!(Level::INFO, "baa");
    {
        let _enter = span.enter();
        event!(Level::INFO, foo = 5, bar = "hello");
    }
    
    cli::run(&args)
}