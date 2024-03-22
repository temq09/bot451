use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use anyhow::anyhow;
use axum::routing::post;
use axum::Router;
use tokio::net::TcpListener;

pub struct RestBackend {
    port: u16,
}

impl RestBackend {
    pub fn new(port: u16) -> Self {
        RestBackend { port }
    }
}

pub async fn init(backend_config: RestBackend) -> anyhow::Result<()> {
    let router = Router::new().route("/v1/requestPageForUser", post(load_page));
    let listener = create_listener(backend_config.port).await?;
    axum::serve(listener, router).await?;
    return Ok(());
}

async fn create_listener(port: u16) -> anyhow::Result<TcpListener> {
    let ipv4addr = Ipv4Addr::new(0, 0, 0, 0);
    let socket_addr = SocketAddr::new(IpAddr::V4(ipv4addr), port);
    return TcpListener::bind(socket_addr)
        .await
        .map_err(|err| anyhow!("Can't bind on given port: {}, error: {}", port, err));
}

async fn load_page() {}
