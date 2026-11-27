//! Serving, and stopping gracefully.

use crate::config::DRAIN;
use axum::Router;
use std::future::Future;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::watch;

/// Serves `app` until `shutdown` resolves, then stops accepting and lets
/// in-flight requests finish for up to `drain`.
///
/// # Errors
///
/// An I/O error from the listener.
pub async fn serve(
    listener: TcpListener,
    app: Router,
    shutdown: impl Future<Output = ()> + Send + 'static,
    drain: Duration,
) -> std::io::Result<()> {
    let (tx, rx) = watch::channel(false);
    let mut graceful = rx.clone();
    let mut deadline = rx;
    drop(tokio::spawn(async move {
        shutdown.await;
        tracing::info!("shutting down: draining in-flight requests for up to {drain:?}");
        let _ = tx.send(true);
    }));
    let server = axum::serve(listener, app).with_graceful_shutdown(async move {
        let _ = graceful.wait_for(|stop| *stop).await;
    });
    tokio::select! {
        result = server => result,
        () = async move {
            let _ = deadline.wait_for(|stop| *stop).await;
            tokio::time::sleep(drain).await;
        } => {
            tracing::warn!("drain deadline reached; exiting with requests still open");
            Ok(())
        },
    }
}

/// Resolves on Ctrl-C or, on Unix, SIGTERM.
pub async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut s) => {
                let _ = s.recv().await;
            },
            Err(e) => {
                tracing::error!("cannot listen for SIGTERM: {e}");
                std::future::pending::<()>().await;
            },
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}

/// [`serve`] with the standard 25-second drain.
///
/// # Errors
///
/// As [`serve`].
pub async fn serve_until_signal(listener: TcpListener, app: Router) -> std::io::Result<()> {
    serve(listener, app, shutdown_signal(), DRAIN).await
}
