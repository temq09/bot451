use std::path::Path;
use std::sync::Arc;

use anyhow::{bail, Context};
use clap::Parser;

use api::{PagePersistent, PageUploader, PageWorker};
use botbackend::parallel_page_worker::ParallelPageWorker;
use rest_backend::{init, RestBackend};
use sqlite::persistent_page_worker::PersistentPageWorker;
use sqlite::postgres_persistent::PostgresPersistent;
use sqlite::sqlite_persistent::init_db;

use crate::backend_args::BackendArgs;
use crate::teloxide_bot::TeloxidePageUploader;

mod backend_args;
mod teloxide_bot;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let backend_args = BackendArgs::parse();
    let persistence = create_persistent(&backend_args).await?;
    let config = RestBackend::new(
        8080,
        create_loader(
            persistence.clone(),
            &backend_args.singlefile_cli,
            &backend_args.work_dir,
        ),
        create_uploader(),
        persistence,
    );
    init(config).await
}

fn create_loader(
    cache: Arc<dyn PagePersistent>,
    singlefile_cli: &str,
    work_dir: &str,
) -> impl PageWorker {
    let network_page_worker =
        ParallelPageWorker::new(work_dir.to_string(), singlefile_cli.to_string());
    PersistentPageWorker::new(cache, Box::new(network_page_worker))
}

fn create_uploader() -> impl PageUploader {
    TeloxidePageUploader::new_from_env()
}

async fn create_persistent(args: &BackendArgs) -> anyhow::Result<Arc<dyn PagePersistent>> {
    if let Some(url) = args.pg_url.as_ref() {
        create_postgres(url, args).await
    } else {
        create_sqlite(args).await
    }
}

async fn create_postgres(
    host: &str,
    args: &BackendArgs,
) -> anyhow::Result<Arc<dyn PagePersistent>> {
    let password = args
        .pg_password
        .as_ref()
        .context("Password must be set when postgres is used")?;
    let username = args
        .pg_user
        .as_ref()
        .context("Username must be set when postgres is used")?;
    let db = args
        .pg_database
        .as_ref()
        .context("Database must be set when postgres is used")?;

    let persistent = PostgresPersistent::connect(username, password, db, host).await?;
    Ok(Arc::new(persistent))
}

async fn create_sqlite(args: &BackendArgs) -> anyhow::Result<Arc<dyn PagePersistent>> {
    let work_dir = get_sqlite_db_path(args.work_dir.as_str()).await?;
    let persistent = init_db(work_dir.to_string()).await?;
    Ok(Arc::new(persistent))
}

async fn get_sqlite_db_path(work_dir: &str) -> anyhow::Result<String> {
    let work_dir_path = Path::new(work_dir);
    if !work_dir_path.exists() {
        tokio::fs::create_dir_all(work_dir_path).await?;
    }
    if !work_dir_path.is_dir() {
        bail!("Work dir {} is not a folder", work_dir)
    }

    let mut dir = String::from(work_dir);
    dir.push_str("/bot_db.db");

    tokio::fs::File::create(dir.clone()).await?;

    Ok(dir)
}
