use async_trait::async_trait;

use api::{PageData, PagePersistent, PageResult, PageWorker};

struct PersistentPageWorker {
    pub(crate) storage: Box<dyn PagePersistent>,
    pub(crate) fallback_worker: Box<dyn PageWorker>,
}

#[async_trait]
impl PageWorker for PersistentPageWorker {
    async fn submit_page_generation(&self, page_data: PageData) -> anyhow::Result<PageResult> {
        let persistent_page_data = self
            .storage
            .get(page_data.url.as_str())
            .await
            .unwrap_or(None);

        match persistent_page_data {
            None => self.fallback_worker.submit_page_generation(page_data).await,
            Some(persistent_page) => Ok(PageResult::TelegramId(persistent_page.telegram_file_id)),
        }
    }
}
