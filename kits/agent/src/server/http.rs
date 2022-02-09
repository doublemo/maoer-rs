use std::net::{SocketAddr};
use std::sync::Arc;
use std::io;
use futures::TryStreamExt;
use hyper::service::{make_service_fn, service_fn};
use tokio::net::{TcpSocket, TcpStream};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use crate::server::{Context, self};
use crate::server::socket::{is_dual_stack_addr, socket_bind_dual_stack};

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
    
    
    let listener = socket.listen(1024)?;
    let std_listener = listener.into_std()?;
    let service = make_service_fn(|_| async { Ok::<_, hyper::Error>(service_fn(handle_connection)) });
    match Server::from_tcp(std_listener) {
        Ok(server) => {
            let s = server.serve(service);
            tracing::info!("Listening on: http://{}", addr);
            match s.await {
                Ok(()) => Ok(()),
                Err(_e) => Ok(())
            }
        },
        Err(e) => {
            Ok(())
        }
    }
}

async fn handle_connection(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        // Serve some instructions at /
        (&Method::GET, "/") => Ok(Response::new(Body::from(
            "Try POSTing data to /echo such as: `curl localhost:3000/echo -XPOST -d 'hello world'`",
        ))),

        // Simply echo the body back to the client.
        (&Method::POST, "/echo") => Ok(Response::new(req.into_body())),

        // Convert to uppercase before sending back to client using a stream.
        (&Method::POST, "/echo/uppercase") => {
            let chunk_stream = req.into_body().map_ok(|chunk| {
                chunk
                    .iter()
                    .map(|byte| byte.to_ascii_uppercase())
                    .collect::<Vec<u8>>()
            });
            Ok(Response::new(Body::wrap_stream(chunk_stream)))
        }

        // Reverse the entire body before sending back to the client.
        //
        // Since we don't know the end yet, we can't simply stream
        // the chunks as they arrive as we did with the above uppercase endpoint.
        // So here we do `.await` on the future, waiting on concatenating the full body,
        // then afterwards the content can be reversed. Only then can we return a `Response`.
        (&Method::POST, "/echo/reversed") => {
            let whole_body = hyper::body::to_bytes(req.into_body()).await?;

            let reversed_body = whole_body.iter().rev().cloned().collect::<Vec<u8>>();
            Ok(Response::new(Body::from(reversed_body)))
        }

        // Return the 404 Not Found for other routes.
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}