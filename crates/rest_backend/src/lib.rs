use std::net::Ipv4Addr;

use axum::routing::post;
use axum::Router;
use tokio::net::TcpListener;

pub struct RestBackend {
    port: u8,
}

pub async fn init(backend_config: RestBackend) -> anyhow::Result<()> {
    let router = Router::new().route("v1/requestPageForUser", post(load_page));

    let ipv4addr = Ipv4Addr::new(0, 0, 0, backend_config.port);
    let listener = TcpListener::bind(ipv4addr.to_string()).await?;

    axum::serve(listener, router).await?;
    return Ok(());
}

async fn load_page() {}
