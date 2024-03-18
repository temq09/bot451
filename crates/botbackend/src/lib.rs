use std::fs::File;
use async_trait::async_trait;

pub mod parallel_page_worker;

pub struct PageData {
    url: String,
}

impl PageData {
    pub fn from_url(url: String) -> Self {
        PageData {
            url
        }
    }
}

#[async_trait]
pub trait PageWorker {
    async fn submit_page_generation(&self, page_data: PageData) -> anyhow::Result<File>;
}
