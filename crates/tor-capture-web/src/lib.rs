//! Web interface with Axum and HTMX.

pub mod routes;
pub mod state;
pub mod templates;

pub use routes::create_router;
pub use state::AppState;

use std::net::SocketAddr;
use tokio::net::TcpListener;
use tor_capture_core::WebConfig;

/// Start the web server.
pub async fn start_server(state: AppState, config: &WebConfig) -> anyhow::Result<()> {
    let app = create_router(state);

    let addr: SocketAddr = format!("{}:{}", config.bind_address, config.port)
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid bind address: {}", e))?;

    tracing::info!("Starting web server on http://{}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
