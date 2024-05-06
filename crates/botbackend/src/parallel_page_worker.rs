use std::path::PathBuf;

use async_trait::async_trait;
use nanoid::nanoid;
use tokio::process::Command;

use api::{PageData, PageResult, PageWorker};

pub struct ParallelPageWorker {
    working_dir: String,
    singlefile_cli_path: String,
}

impl ParallelPageWorker {
    pub fn new(working_dir: String, singlefile_cli_path: String) -> Self {
        ParallelPageWorker {
            working_dir,
            singlefile_cli_path,
        }
    }
}

#[async_trait]
impl PageWorker for ParallelPageWorker {
    async fn submit_page_generation(&self, page_data: PageData) -> anyhow::Result<PageResult> {
        let mut file_path = PathBuf::from(self.working_dir.to_owned());
        file_path.push(nanoid!());
        file_path.set_extension("html");
        let path_str = file_path.to_str().unwrap().to_owned();
        let result = PageResult::FilePath(path_str.to_owned());
        let output = Command::new(&self.singlefile_cli_path)
            .arg("--remove-saved-date")
            .arg(page_data.url)
            .arg(path_str)
            .output()
            .await?;

        if !output.status.success() {
            return Err(anyhow::Error::msg("Can't execute command"));
        }

        return Ok(result);
    }
}
