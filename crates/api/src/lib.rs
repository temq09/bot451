use async_trait::async_trait;

pub struct PageData {
    pub url: String,
    pub user_id: String,
}

pub enum PageResult {
    FilePath(String),
    Noop,
}

impl PageData {
    pub fn from_url(url: String, user_id: String) -> Self {
        PageData { url, user_id }
    }
}

#[async_trait]
pub trait PageWorker: Sync + Send {
    async fn submit_page_generation(&self, page_data: PageData) -> anyhow::Result<PageResult>;
}

#[async_trait]
pub trait PageUploader: Sync + Send {
    async fn send_page(&self, chat_id: String, page_result: PageResult) -> anyhow::Result<()>;
}
