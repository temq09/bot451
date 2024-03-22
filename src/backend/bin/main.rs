use rest_backend::{init, RestBackend};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = RestBackend::new(8080);
    init(config).await
}
