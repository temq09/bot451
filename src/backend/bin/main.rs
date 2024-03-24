use api::PageWorker;
use botbackend::parallel_page_worker::ParallelPageWorker;
use rest_backend::{init, RestBackend};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = RestBackend::new(8080, create_loader());
    init(config).await
}

fn create_loader() -> impl PageWorker {
    ParallelPageWorker::new("/Users/artemushakov/prog/tmp/singlefile".to_string())
}
