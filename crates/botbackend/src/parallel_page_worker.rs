use std::path::PathBuf;
use std::process::Command;

use async_trait::async_trait;

use crate::{PageData, PageResult, PageWorker};

pub struct ParallelPageWorker {}

impl ParallelPageWorker {
    pub fn new() -> Self {
        return ParallelPageWorker {};
    }
}

#[async_trait]
impl PageWorker for ParallelPageWorker {
    async fn submit_page_generation(&self, page_data: PageData) -> anyhow::Result<PageResult> {
        let mut file_path = PathBuf::from("/Users/artemushakov/prog/tmp/singlefile");
        file_path.push("page_name");
        file_path.set_extension("html");
        let path_str = file_path.to_str().unwrap().to_owned();
        println!("Html page path: {}", path_str);
        let result = PageResult::FilePath(path_str.to_owned());
        let output = Command::new("/Users/artemushakov/prog/tmp/singlefile/singlefile")
            .arg(page_data.url)
            .arg(path_str)
            .output()?;

        if !output.status.success() {
            return Err(anyhow::Error::msg("Can't execute command"));
        }

        return Ok(result);
    }
}
