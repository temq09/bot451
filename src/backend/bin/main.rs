use std::sync::Arc;

use api::{PagePersistent, PageUploader, PageWorker};
use botbackend::parallel_page_worker::ParallelPageWorker;
use rest_backend::{init, RestBackend};
use sqlite::persistent_page_worker::PersistentPageWorker;
use sqlite::sqlite_persistent::in_memory_db;

use crate::teloxide_bot::TeloxidePageUploader;

mod teloxide_bot;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let persistence = Arc::new(create_persistent().await?);
    let config = RestBackend::new(
        8080,
        create_loader(persistence.clone()),
        create_uploader(),
        persistence,
    );
    init(config).await
}

fn create_loader(cache: Arc<impl PagePersistent + 'static>) -> impl PageWorker {
    let network_page_worker =
        ParallelPageWorker::new("/Users/artemushakov/prog/tmp/singlefile".to_string());
    PersistentPageWorker::new(cache, Box::new(network_page_worker))
}

fn create_uploader() -> impl PageUploader {
    TeloxidePageUploader::new_from_env()
}

async fn create_persistent() -> anyhow::Result<impl PagePersistent> {
    in_memory_db().await
}
