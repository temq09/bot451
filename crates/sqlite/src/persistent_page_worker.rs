use std::sync::Arc;

use async_trait::async_trait;

use api::{PageData, PagePersistent, PageResult, PageWorker};

pub struct PersistentPageWorker {
    pub(crate) storage: Arc<dyn PagePersistent>,
    pub(crate) fallback_worker: Box<dyn PageWorker>,
}

impl PersistentPageWorker {
    pub fn new(
        storage: Arc<impl PagePersistent + 'static>,
        fallback_worker: Box<dyn PageWorker>,
    ) -> Self {
        PersistentPageWorker {
            storage,
            fallback_worker,
        }
    }
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

#[cfg(test)]
mod tests {
    use api::{PageData, PageInfo, PageResult, PageWorker};

    use crate::persistent_page_worker::test_impl::{MockPagePersistent, MockPageWorker};
    use crate::persistent_page_worker::PersistentPageWorker;

    #[sqlx::test]
    async fn test_no_item_in_cache() -> anyhow::Result<()> {
        let persistent = MockPagePersistent::new();
        let mut page_worker = Box::new(MockPageWorker::new());
        page_worker.data_storage.insert(
            "url_1".to_string(),
            PageResult::TelegramId("id_1".to_string()),
        );
        let worker = PersistentPageWorker::new(Box::new(persistent), page_worker);

        let result = worker
            .submit_page_generation(PageData::from_url("url_1".to_string(), "id".to_string()))
            .await?;

        assert_eq!(result, PageResult::TelegramId("id_1".to_string()));

        Ok(())
    }

    #[sqlx::test]
    async fn test_item_in_cache() -> anyhow::Result<()> {
        let mut persistent = MockPagePersistent::new();
        persistent.data_storage.insert(
            "url_1".to_string(),
            PageInfo {
                telegram_file_id: "telegram_id".to_string(),
                file_hash: "hash".to_string(),
                page_url: "url_1".to_string(),
                timestamp_ms: 123,
            },
        );
        let mut page_worker = Box::new(MockPageWorker::new());
        page_worker.data_storage.insert(
            "url_1".to_string(),
            PageResult::FilePath("/some/path".to_string()),
        );
        let worker = PersistentPageWorker::new(Box::new(persistent), page_worker);

        let result = worker
            .submit_page_generation(PageData::from_url("url_1".to_string(), "id".to_string()))
            .await?;

        assert_eq!(result, PageResult::TelegramId("telegram_id".to_string()));

        Ok(())
    }
}

#[cfg(test)]
mod test_impl {
    use std::collections::HashMap;

    use anyhow::{bail, Error};
    use async_trait::async_trait;

    use api::{PageData, PageInfo, PagePersistent, PageResult, PageWorker};

    pub struct MockPagePersistent {
        pub data_storage: HashMap<String, PageInfo>,
    }

    impl MockPagePersistent {
        pub(crate) fn new() -> Self {
            MockPagePersistent {
                data_storage: HashMap::new(),
            }
        }
    }

    #[async_trait]
    impl PagePersistent for MockPagePersistent {
        async fn save(&self, page_info: &PageInfo) -> anyhow::Result<()> {
            bail!("Not supported")
        }

        async fn get(&self, page_url: &str) -> anyhow::Result<Option<PageInfo>> {
            Ok(self.data_storage.get(page_url).map(|result| result.clone()))
        }
    }

    pub struct MockPageWorker {
        pub data_storage: HashMap<String, PageResult>,
    }

    impl MockPageWorker {
        pub fn new() -> Self {
            MockPageWorker {
                data_storage: HashMap::new(),
            }
        }
    }

    #[async_trait]
    impl PageWorker for MockPageWorker {
        async fn submit_page_generation(&self, page_data: PageData) -> anyhow::Result<PageResult> {
            self.data_storage
                .get(page_data.url.as_str())
                .map(|result| result.clone())
                .ok_or(Error::msg("Wrong result"))
        }
    }
}
