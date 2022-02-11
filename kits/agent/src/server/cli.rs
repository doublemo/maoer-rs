use clap::Parser;
use std::fmt;
use std::process;
use std::sync::Arc;
use futures::future::{self, Either};
use tokio::runtime::Builder;
use crate::server;
use crate::server::Context;
use maoer_cores::monitor;

#[derive(Parser, Debug)]
#[clap(name = "网关服务器")]
#[clap(about,author,version)]
pub struct Args {
    /// 配置文件地址
    #[clap(short, long, default_value_t=String::from("configs/agent.toml"))]
    pub config:String,

    /// 设置调试等级
    #[clap(short, long, parse(from_occurrences))]
    pub debug: usize,

    /// 日志等级
    #[clap(short, long, default_value_t=String::from("info"))]
    pub log_level:String
}

impl fmt::Display for Args {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " 
                    config: {},
                    debug:{},
                    log_level: {}
                  ", 
                    &self.config, 
                    &self.debug,
                    &self.log_level
        )
    }
}

/// Program entrance `main`
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = Arc::new(Context::new());
    let runtime = Builder::new_multi_thread()
                                    .enable_all()
                                    .build().expect("create tokio Runtime");

                                    

    runtime.block_on(async move {
        let abort_signal = monitor::create_signal_monitor();
        let servers = server::create(ctx).await.unwrap();
        let serv = servers.run();
        tokio::pin!(abort_signal);
        tokio::pin!(serv);
        match future::select(serv, abort_signal).await {
            // Server future resolved without an error. This should never happen.
            Either::Left((Ok(..), ..)) => {
                eprintln!("server exited unexpectedly");
                process::exit(1);
            }
            // Server future resolved with error, which are listener errors in most cases
            Either::Left((Err(err), ..)) => {
                eprintln!("server aborted with {}", err);
                process::exit(2);
            }
            // The abort signal future resolved. Means we should just exit.
            Either::Right(_) => (),
        }
    });
    Ok(())
}