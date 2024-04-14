use async_trait::async_trait;
use time::OffsetDateTime;

pub struct PageData {
    pub url: String,
    pub user_id: String,
}

#[derive(Clone, PartialEq, Debug)]
pub enum PageResult {
    FilePath(String),
    TelegramId(String),
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
    async fn send_page(
        &self,
        chat_id: String,
        page_result: &PageResult,
    ) -> anyhow::Result<Option<String>>;
}

#[derive(Debug, PartialEq, Clone)]
pub struct PageInfo {
    pub telegram_file_id: String,
    pub file_hash: String,
    pub page_url: String,
    pub timestamp_ms: OffsetDateTime,
}

#[async_trait]
pub trait PagePersistent: Sync + Send {
    async fn save(&self, page_info: &PageInfo) -> anyhow::Result<()>;
    async fn get(&self, page_url: &str) -> anyhow::Result<Option<PageInfo>>;
}
