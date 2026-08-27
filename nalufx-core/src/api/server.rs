//! The HTTP server.
//!
//! A hyper connection-accept loop. This is the whole of what the framework
//! previously provided: bind a listener, serve each connection, dispatch to
//! the router.

use crate::api::router::route;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use log::{error, info};
use reqwest::Client;
use std::net::SocketAddr;
use tokio::net::TcpListener;

/// Serves requests until the process is terminated.
///
/// # Arguments
///
/// * `addr` - The address to bind.
///
/// # Returns
///
/// Never returns under normal operation.
///
/// # Errors
///
/// Returns an error if the listener cannot bind to `addr`. Errors on an
/// individual connection are logged and do not stop the server.
pub async fn serve(addr: SocketAddr) -> std::io::Result<()> {
    serve_on(TcpListener::bind(addr).await?).await
}

/// Serves requests on an already-bound listener.
///
/// Splitting this from [`serve`] lets a test bind port 0 and learn the
/// assigned port before the loop starts, which binding inside the loop
/// would make impossible.
///
/// # Arguments
///
/// * `listener` - A bound TCP listener.
///
/// # Returns
///
/// Never returns under normal operation.
///
/// # Errors
///
/// Errors on an individual connection are logged rather than returned, so
/// this only fails if the listener itself becomes unusable.
pub async fn serve_on(listener: TcpListener) -> std::io::Result<()> {
    if let Ok(addr) = listener.local_addr() {
        info!("Listening on http://{addr}");
    }

    // One client is shared across connections so its pool is reused.
    let client = Client::new();

    loop {
        let (stream, peer) = match listener.accept().await {
            Ok(accepted) => accepted,
            Err(err) => {
                // A failed accept concerns one peer, not the listener, so
                // the loop continues rather than tearing down the server.
                error!("Error accepting connection: {err}");
                continue;
            },
        };

        let client = client.clone();
        // The connection task is detached: it owns its stream and the
        // accept loop must not wait on it.
        drop(tokio::spawn(async move {
            let service = service_fn(move |req| route(client.clone(), req));
            if let Err(err) =
                http1::Builder::new().serve_connection(TokioIo::new(stream), service).await
            {
                error!("Error serving {peer}: {err}");
            }
        }));
    }
}
