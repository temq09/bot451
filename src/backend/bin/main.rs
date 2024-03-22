use rest_backend::{init, RestBackend};

#[tokio::main]
async fn main() {
    let config = RestBackend::new(8080);
    init(config).await.expect("Server failed");
}
