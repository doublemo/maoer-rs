pub mod cli;
mod http;
mod socket;
mod websocket;
mod context;

pub use self::context::Context;
use std::io;
use std::sync::Arc;
use std::time::Instant;
use std::net::{SocketAddr, IpAddr, Ipv6Addr};
use futures::future::BoxFuture;
use futures::stream::{FuturesUnordered, StreamExt};
use futures::FutureExt;
use tracing;


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
    let vfut = FuturesUnordered::new();
    let socket_ctx = ctx.clone();
    vfut.push(async move { socket::run(socket_ctx, SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), 8088)).await}.boxed());

    let websocket_ctx = ctx.clone();
    vfut.push(async move { websocket::run(websocket_ctx, SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), 8098)).await}.boxed());
    
    let http_ctx = ctx.clone();
    vfut.push(async move { http::run(http_ctx, SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 0)), 8099)).await}.boxed());
    tracing::info!("time: {:?}", bf.elapsed());
    Ok(Server{vfut})
}