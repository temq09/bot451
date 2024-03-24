use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use anyhow::anyhow;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use tokio::net::TcpListener;

use api::{PageData, PageWorker};

use crate::error::AppError;

mod error;

pub struct RestBackend {
    port: u16,
    page_loader: Arc<dyn PageWorker>,
}

impl RestBackend {
    pub fn new(port: u16, page_loader: impl PageWorker + 'static) -> Self {
        RestBackend {
            port,
            page_loader: Arc::new(page_loader),
        }
    }
}

pub async fn init(backend_config: RestBackend) -> anyhow::Result<()> {
    let router = Router::new()
        .route("/v1/requestPageForUser", post(load_page))
        .with_state(backend_config.page_loader);
    let listener = create_listener(backend_config.port).await?;
    axum::serve(listener, router).await?;
    return Ok(());
}

async fn create_listener(port: u16) -> anyhow::Result<TcpListener> {
    let ipv4addr = Ipv4Addr::new(0, 0, 0, 0);
    let socket_addr = SocketAddr::new(IpAddr::V4(ipv4addr), port);
    return TcpListener::bind(socket_addr)
        .await
        .map_err(|err| anyhow!("Can't bind on the given port: {}, error: {}", port, err));
}

async fn load_page(
    State(page_loader): State<Arc<dyn PageWorker>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<(), AppError> {
    println!("Load page request for {}", payload);
    let user_id = payload["user_id"]
        .as_str()
        .ok_or(AppError::BadRequest("User id is not set".to_string()))?
        .to_owned();
    let page_url = payload["page_url"]
        .as_str()
        .ok_or(AppError::BadRequest("User id is not set".to_string()))?
        .to_owned();

    page_loader
        .submit_page_generation(PageData::from_url(page_url, user_id))
        .await?;

    return Ok(());
}
