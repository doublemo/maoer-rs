use std::net::{SocketAddr};
use std::io;
use std::time::Duration;
use std::sync::Arc;
use tokio::time;
use tokio::net::{TcpSocket, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Error};
use tungstenite::{Message, Result};
use futures::{SinkExt, StreamExt};
use tracing;
use crate::server::Context;
use crate::server::socket::{is_dual_stack_addr, socket_bind_dual_stack};
use crate::peer::Peer;

pub(crate) async fn run(_ctx:Arc<Context>, addr:SocketAddr) -> io::Result<()> {
    let socket = match addr {
        SocketAddr::V4(..) => TcpSocket::new_v4()?,
        SocketAddr::V6(..) => TcpSocket::new_v6()?,
    };

    #[cfg(not(windows))]
    socket.set_reuseaddr(true)?;

    let set_dual_stack = is_dual_stack_addr(&addr);
    if set_dual_stack {
        socket_bind_dual_stack(&socket, &addr, false)?;
    } else {
        socket.bind(addr)?;
    }
    
    tracing::info!("Listening on: {}", addr);
    let listener = socket.listen(1024)?;
    while let Ok((stream, saddr)) = listener.accept().await {
        tokio::spawn(accept_connection(saddr, stream));
    }

    Ok(())
}


async fn accept_connection(addr: SocketAddr, stream: TcpStream) {
    if let Err(e) = handle_connection(addr, stream).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => tracing::error!("Error processing connection: {}", err),
        }
    }
}

async fn handle_connection(addr: SocketAddr, stream: TcpStream) -> Result<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    tracing::debug!("new peer from {}", addr.to_string());
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let mut interval = time::interval(Duration::from_millis(1000));
    let mut peer = Peer::new(addr.to_string()).await;

    loop {
        tokio::select! {
            msg = ws_receiver.next() => {
                match msg {
                    Some(msg) => {
                        let msg = msg?;
                        if msg.is_text() ||msg.is_binary() {
                            ws_sender.send(msg).await?;
                        } else if msg.is_close() {
                            break;
                        }
                    },

                    None => break,
                }
            },
            
            w = peer.rx.recv() => {
                tracing::debug!("write msg: {:?}", w);
                if let Some(r) = w  {
                    ws_sender.send(Message::Binary(r.to_vec())).await?
                }
            },

            _ = interval.tick() => {
                ws_sender.send(Message::Text("tick".to_owned())).await?;
            }
        }
    }

    Ok(())
}