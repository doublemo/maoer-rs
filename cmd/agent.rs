use std::str::FromStr;

use clap::Parser;
use maoer_agent::server::cli;
use maoer_agent::server::cli::Args;
use maoer_agent::config;
use tracing::{self, Level};
use tracing_subscriber::{fmt::format::FmtSpan};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = Args::parse();
    config::init(args.config.as_str());
    let c = config::Configuration::new().unwrap();
    args.debug = if c.debug {
        1
    } else {
        0
    };

    args.log_level = c.log_level.clone();
    let filter_level = match Level::from_str(args.log_level.as_str()) {
        Ok(level) => level,
        Err(_) => Level::INFO
    };

    // 设置日志
    tracing_subscriber::fmt().with_max_level(filter_level)
    .with_span_events(FmtSpan::FULL).init();
    tracing::info!("Starting Agent server");
    tracing::info!("Version: {}", maoer_agent::VERSION);
    tracing::info!("Build At: {}", maoer_agent::BUILD_TIME);
    tracing::info!("Using configuration file: {}", args.config);
    tracing::info!(r#"            ___  ___  ___  _____ ___________ "#);
    tracing::info!(r#"            |  \/  | / _ \|  _  |  ___| ___ \"#);
    tracing::info!(r#"            | .  . |/ /_\ \ | | | |__ | |_/ /"#);
    tracing::info!(r#"            | |\/| ||  _  | | | |  __||    / "#);
    tracing::info!(r#"            | |  | || | | \ \_/ / |___| |\ \"#);
    tracing::info!(r#"            \_|  |_/\_| |_/\___/\____/\_| \_|"#);
    tracing::info!("");
    tracing::info!("            https://github.com/doublemo/maoer-rs ");
    config::show();

    cli::run(&args)
}

