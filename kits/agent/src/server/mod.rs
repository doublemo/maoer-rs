pub mod cli;
mod http;
mod socket;
mod websocket;
mod context;

pub use self::context::Context;
use std::io;
use std::sync::Arc;
use std::time::Instant;
use std::net::{SocketAddr, IpAddr};
use futures::future::BoxFuture;
use futures::stream::{FuturesUnordered, StreamExt};
use futures::FutureExt;
use tracing;
use crate::config::Configuration;

/// 服务器定义
pub struct Server {
    vfut: FuturesUnordered<BoxFuture<'static, io::Result<()>>>,
}

impl Server {
    pub async fn run(self) -> io::Result<()> {
        let (res, _) = self.vfut.into_future().await;
        res.unwrap()
    }
 }

pub async fn create(ctx:Arc<Context>) -> io::Result<Server>{
    let bf = Instant::now();
    let conf = Configuration::new().unwrap();
    let vfut = FuturesUnordered::new();

    // socket
    if let Some(socket_config) = conf.socket {
        let socket_ctx = ctx.clone();
        let socket_addr = socket_config.addr.parse::<IpAddr>().unwrap();
        vfut.push(async move { socket::run(socket_ctx, SocketAddr::new(socket_addr, socket_config.port), socket_config.backlog).await}.boxed());
    }

    // websocket
    if let Some(websocket_config) = conf.websocket {
        let websocket_addr = websocket_config.addr.parse::<IpAddr>().unwrap();
        let ctx_1 = ctx.clone();
        vfut.push(async move { websocket::run(ctx_1, SocketAddr::new(websocket_addr, websocket_config.port)).await}.boxed());
    }

    // http
    if let Some(http_config) = conf.http {
        let http_addr = http_config.addr.parse::<IpAddr>().unwrap();
        let http_ctx = ctx.clone();
        vfut.push(async move { http::run(http_ctx, SocketAddr::new(http_addr, http_config.port)).await}.boxed());
    }
    
    tracing::info!("time: {:?}", bf.elapsed());
    Ok(Server{vfut})
}