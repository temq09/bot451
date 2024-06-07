use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, bail, Context};
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
    let work_dir = create_file_if_needed(args.work_dir.as_ref(), "/bot_db.db").await?;
    let persistent = init_db(work_dir.to_string()).await?;
    Ok(Arc::new(persistent))
}

/// Check if file with the file name exist in the given folder.
/// Returns fill path to the file
async fn create_file_if_needed(work_dir: &Path, file_name: &str) -> anyhow::Result<String> {
    if !work_dir.exists() {
        tokio::fs::create_dir_all(work_dir).await?;
    }
    if !work_dir.is_dir() {
        bail!("Work dir {} is not a folder", work_dir.display())
    }

    let file_path = work_dir.join(file_name);
    let path = file_path
        .to_str()
        .ok_or(anyhow!("Can't convert path to str"))?;

    if !file_path.exists() {
        tokio::fs::File::create(file_path.clone()).await?;
    }

    Ok(String::from(path))
}

#[cfg(test)]
mod test {
    use std::env::temp_dir;
    use std::path::Path;

    use crate::create_file_if_needed;

    #[tokio::test]
    async fn test_create_file() -> anyhow::Result<()> {
        let temp_dir = temp_dir();
        let file_name = "test.db";

        let full_path = create_file_if_needed(temp_dir.as_ref(), file_name).await?;
        let actual_path = Path::new(&full_path);

        assert_eq!(temp_dir.join(file_name), actual_path);
        assert!(actual_path.is_file());
        assert!(actual_path.exists());

        tokio::fs::write(actual_path, "test_data").await?;
        let full_path = create_file_if_needed(temp_dir.as_ref(), file_name).await?;
        let actual_path = Path::new(&full_path);
        let content = tokio::fs::read(actual_path).await?;
        assert_eq!(content, "test_data".as_bytes());

        Ok(())
    }
}
