use api::{PageUploader, PageWorker};
use botbackend::parallel_page_worker::ParallelPageWorker;
use rest_backend::{init, RestBackend};

use crate::teloxide_bot::TeloxidePageUploader;

mod teloxide_bot;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = RestBackend::new(8080, create_loader(), create_uploader());
    init(config).await
}

fn create_loader() -> impl PageWorker {
    ParallelPageWorker::new("/Users/artemushakov/prog/tmp/singlefile".to_string())
}

fn create_uploader() -> impl PageUploader {
    TeloxidePageUploader::new_from_env()
}
