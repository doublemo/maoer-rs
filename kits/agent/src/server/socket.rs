use std::net::{SocketAddr};
use std::time::Duration;
use std::sync::Arc;
use std::io::{Cursor, ErrorKind};
use futures::StreamExt;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::time::{self, Instant};
use tokio::net::{TcpSocket, TcpStream};
use tokio_util::codec::{LengthDelimitedCodec};
use tracing;
use bytes::{BytesMut, BufMut};
use socket2::{Socket as Socket2, SockAddr};
use crate::server::Context;
use crate::peer::{Peer, handle_msg};

pub(crate) async fn run(_ctx:Arc<Context>, addr:SocketAddr, backlog:u32) -> io::Result<()> {
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
    let listener = socket.listen(backlog)?;
    while let Ok((stream, saddr)) = listener.accept().await {
        tokio::spawn(accept_connection(saddr, stream));
    }

    Ok(())
}


async fn accept_connection(addr: SocketAddr, stream: TcpStream) {
    if let Err(e) = handle_connection(addr, stream).await {
        tracing::error!("an error occurred while processing messages; error = {:?}", e);
    }
}

async fn handle_connection(addr: SocketAddr, stream: TcpStream) -> io::Result<()> {
    tracing::debug!("new peer from {}", addr.to_string());
    let mut framed= LengthDelimitedCodec::builder()
                    .length_field_offset(0)
                    .length_field_length(2)
                    .length_adjustment(0)
                    .new_framed(stream);

    let mut peer = Peer::new(addr.to_string()).await;
    let sleep = time::sleep(time::Duration::from_millis(100));
    tokio::pin!(sleep);

    loop {
        tokio::select! {
            _ = &mut sleep => {
                sleep.as_mut().reset(Instant::now() + Duration::from_millis(100));
            },

            result  =  framed.next() =>  match result {
                Some(Ok(msg)) => {
                    tracing::debug!("recv client msg for {}, msg: {:?}", peer.get_id(), msg);
                    handle_msg(peer.as_ref());

                    let mut cur = Cursor::new(msg);
                    let x = cur.read_u8().await;
                    println!("-------<u8>----- {:?}", x);
                    let x = cur.read_u8().await;
                    println!("-------<u8>----- {:?}", x);
                    

                    let mut wmsg = BytesMut::with_capacity(16);
                    wmsg.put_u16(10u16);
                    match peer.send(wmsg).await {
                        Ok(()) => {},
                        Err(_) => {}
                    }
                },

                Some(Err(ref e)) if e.kind() == io::ErrorKind::WouldBlock => continue,
                Some(Err(e)) => {
                    return Err(e);
                },
                None => break
            },

            w = peer.rx.recv() => {
                tracing::debug!("write msg: {:?}", w);
                if let Some(r) = w  {
                    match framed.get_mut().write(&r).await {
                        Ok(_) => {},
                        Err(e) => tracing::error!("an error occurred while writeing messages; error = {:?}", e)
                    }
                }
            }
        }
    }

    tracing::debug!("{} on shutdown", peer.get_id());
    Ok(())
}


/// Check if `SocketAddr` could be used for creating dual-stack sockets
pub(crate) fn is_dual_stack_addr(addr: &SocketAddr) -> bool {
    if let SocketAddr::V6(ref v6) = *addr {
        v6.ip().is_unspecified()
    } else {
        false
    }
}

/// Try to call `bind()` with dual-stack enabled.
///
/// Users have to ensure that `addr` is a dual-stack inbound address (`::`) when `ipv6_only` is `false`.
#[cfg(unix)]
pub(crate) fn socket_bind_dual_stack<S>(socket: &S, addr: &SocketAddr, ipv6_only: bool) -> io::Result<()> 
where S:std::os::unix::io::AsRawFd,
{
    use std::os::unix::prelude::{FromRawFd, IntoRawFd};

    let fd = socket.as_raw_fd();

    let sock = unsafe { Socket::from_raw_fd(fd) };
    let result = socket_bind_dual_stack_inner(&sock, addr, ipv6_only);
    sock.into_raw_fd();

    result
}

/// Try to call `bind()` with dual-stack enabled.
///
/// Users have to ensure that `addr` is a dual-stack inbound address (`::`) when `ipv6_only` is `false`.
#[cfg(windows)]
pub(crate)  fn socket_bind_dual_stack<S>(socket: &S, addr: &SocketAddr, ipv6_only: bool) -> io::Result<()>
where
    S: std::os::windows::io::AsRawSocket,
{
    use std::os::windows::prelude::{FromRawSocket, IntoRawSocket};

    let handle = socket.as_raw_socket();

    let sock = unsafe { Socket2::from_raw_socket(handle) };
    let result = socket_bind_dual_stack_inner(&sock, addr, ipv6_only);
    sock.into_raw_socket();
    result
}

pub(crate)  fn socket_bind_dual_stack_inner(socket: &Socket2, addr: &SocketAddr, ipv6_only: bool) -> io::Result<()> {
    let saddr = SockAddr::from(*addr);

    if ipv6_only {
        // Requested to set IPV6_V6ONLY
        socket.set_only_v6(true)?;
        socket.bind(&saddr)?;
    } else {
        if let Err(err) = socket.set_only_v6(false) {
            tracing::warn!("failed to set IPV6_V6ONLY: false for listener, error: {}", err);
            // This is not a fatal error, just warn and skip
        }

        match socket.bind(&saddr) {
            Ok(..) => {}
            Err(ref err) if err.kind() == ErrorKind::AddrInUse => {
                // This is probably 0.0.0.0 with the same port has already been occupied
                tracing::debug!("0.0.0.0:{} may have already been occupied, retry with IPV6_V6ONLY", addr.port());
                if let Err(err) = socket.set_only_v6(true) {
                    tracing::warn!("failed to set IPV6_V6ONLY: true for listener, error: {}", err);

                    // This is not a fatal error, just warn and skip
                }
                socket.bind(&saddr)?;
            }
            Err(err) => return Err(err),
        }
    }

    Ok(())
}
