use std::env::temp_dir;
use std::fs::File;
use std::process::Command;
use async_trait::async_trait;

use crate::{PageData, PageWorker};

pub struct ParallelPageWorker {}

impl ParallelPageWorker {
    pub fn new() -> impl PageWorker {
        return ParallelPageWorker {};
    }
}

#[async_trait]
impl PageWorker for ParallelPageWorker {
    async fn submit_page_generation(&self, page_data: PageData) -> anyhow::Result<File> {
        let mut file_path = temp_dir();
        file_path.push(&page_data.url);
        file_path.set_extension("html");
        let path_str = file_path.to_str().unwrap().to_owned();
        let file = File::create(file_path)?;
        // let output = Command::new("single-file")
        let output = Command::new("/Users/artemushakov/prog/tmp/singlefile/singlefile")
            .arg(page_data.url)
            .arg(path_str)
            .output()?;

        if !output.status.success() {
            return Err(anyhow::Error::msg("Can't execute command"));
        }

        return Ok(file);
    }
}
