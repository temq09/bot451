use std::cell::Cell;
use std::ops::Sub;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use time::PrimitiveDateTime;

use api::{PageData, PageInfo, PagePersistent, PageResult, PageWorker};
use utils::hash::make_hash_for_file;

pub struct PersistentPageWorker {
    storage: Arc<dyn PagePersistent>,
    fallback_worker: Box<dyn PageWorker>,
}

impl PersistentPageWorker {
    pub fn new(storage: Arc<dyn PagePersistent>, fallback_worker: Box<dyn PageWorker>) -> Self {
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
            Some(persistent_page) => {
                download_for_existing_page(persistent_page, page_data, &self.fallback_worker).await
            }
        }
    }
}

async fn download_for_existing_page(
    persistent_page: PageInfo,
    page_data: PageData,
    fallback_worker: &Box<dyn PageWorker>,
) -> anyhow::Result<PageResult> {
    if is_expired(&persistent_page.timestamp_ms) {
        let new_page = fallback_worker.submit_page_generation(page_data).await?;
        match new_page {
            PageResult::FilePath(path) => Ok(handle_new_page(path, &persistent_page)),
            _ => Ok(new_page),
        }
    } else {
        Ok(PageResult::TelegramId(
            persistent_page.telegram_file_id.clone(),
        ))
    }
}

fn handle_new_page(path: String, current_page: &PageInfo) -> PageResult {
    let new_page_hash = make_hash_for_file(&path).unwrap_or(String::new());
    if new_page_hash == current_page.file_hash {
        PageResult::TelegramId(current_page.telegram_file_id.clone())
    } else {
        PageResult::FilePath(path)
    }
}

fn is_expired(page_loaded_time: &PrimitiveDateTime) -> bool {
    // 10 minutes
    current_time_secs() - page_loaded_time.assume_utc().unix_timestamp() > 60 * 10
}

#[cfg(test)]
thread_local! {
    static CURRENT_TIMESTAMP: Cell<Option<i64>> = const {Cell::new(Some(0))}
}

fn current_time_secs() -> i64 {
    // replacing with Some(0) to prevent flakiness
    // when the value was not properly set up from a previous test
    #[cfg(test)]
    let timestamp = CURRENT_TIMESTAMP.replace(Some(0)).unwrap();
    #[cfg(not(test))]
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(i64::MAX);
    return timestamp;
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use std::sync::Arc;

    use tempfile::tempdir;
    use time::macros::datetime;

    use api::{PageData, PageInfo, PageResult, PageWorker};

    use crate::persistent_page_worker::test_impl::{MockPagePersistent, MockPageWorker};
    use crate::persistent_page_worker::{
        handle_new_page, is_expired, PersistentPageWorker, CURRENT_TIMESTAMP,
    };

    #[sqlx::test]
    async fn test_no_item_in_cache() -> anyhow::Result<()> {
        let persistent = MockPagePersistent::new();
        let mut page_worker = Box::new(MockPageWorker::new());
        page_worker.data_storage.insert(
            "url_1".to_string(),
            PageResult::TelegramId("id_1".to_string()),
        );
        let worker = PersistentPageWorker::new(Arc::new(persistent), page_worker);

        let result = worker
            .submit_page_generation(PageData::from_url("url_1".to_string()))
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
                timestamp_ms: datetime!(2024-01-02 10:10:10),
            },
        );
        let mut page_worker = Box::new(MockPageWorker::new());
        page_worker.data_storage.insert(
            "url_1".to_string(),
            PageResult::FilePath("/some/path".to_string()),
        );
        let worker = PersistentPageWorker::new(Arc::new(persistent), page_worker);

        let result = worker
            .submit_page_generation(PageData::from_url("url_1".to_string()))
            .await?;

        assert_eq!(result, PageResult::TelegramId("telegram_id".to_string()));

        Ok(())
    }

    #[sqlx::test]
    async fn test_item_in_cache_not_expired() -> anyhow::Result<()> {
        let mut persistent = MockPagePersistent::new();
        persistent.data_storage.insert(
            "url_1".to_string(),
            PageInfo {
                telegram_file_id: "telegram_id".to_string(),
                file_hash: "hash".to_string(),
                page_url: "url_1".to_string(),
                timestamp_ms: datetime!(2024-01-02 10:10:00),
            },
        );
        let mut page_worker = Box::new(MockPageWorker::new());
        page_worker.data_storage.insert(
            "url_1".to_string(),
            PageResult::FilePath("/some/path".to_string()),
        );
        let worker = PersistentPageWorker::new(Arc::new(persistent), page_worker);
        CURRENT_TIMESTAMP.set(Some(1704190510));

        let result = worker
            .submit_page_generation(PageData::from_url("url_1".to_string()))
            .await?;

        assert_eq!(result, PageResult::TelegramId("telegram_id".to_string()));

        Ok(())
    }

    #[sqlx::test]
    async fn test_item_in_cache_expired() -> anyhow::Result<()> {
        let mut persistent = MockPagePersistent::new();
        persistent.data_storage.insert(
            "url_1".to_string(),
            PageInfo {
                telegram_file_id: "telegram_id".to_string(),
                file_hash: "hash".to_string(),
                page_url: "url_1".to_string(),
                timestamp_ms: datetime!(2024-01-02 10:10:01),
            },
        );
        let mut page_worker = Box::new(MockPageWorker::new());
        page_worker.data_storage.insert(
            "url_1".to_string(),
            PageResult::FilePath("/some/path".to_string()),
        );
        let worker = PersistentPageWorker::new(Arc::new(persistent), page_worker);
        CURRENT_TIMESTAMP.set(Some(1704276910));

        let result = worker
            .submit_page_generation(PageData::from_url("url_1".to_string()))
            .await?;

        assert_eq!(result, PageResult::FilePath("/some/path".to_string()));
        Ok(())
    }

    #[test]
    fn test_is_expired() {
        CURRENT_TIMESTAMP.set(Some(0));
        assert!(!is_expired(&datetime!(2024-01-02 00:15:00)));

        CURRENT_TIMESTAMP.set(Some(1704068100));
        assert!(!is_expired(&datetime!(2024-01-01 00:15:00)));

        CURRENT_TIMESTAMP.set(Some(1704070800));
        assert!(is_expired(&datetime!(2024-01-01 00:15:00)));

        CURRENT_TIMESTAMP.set(Some(i64::MAX));
        assert!(is_expired(&datetime!(2024-01-01 00:15:00)));
    }

    #[test]
    fn test_handle_new_page() -> anyhow::Result<()> {
        let tmpdir = tempdir()?;
        let file_path = tmpdir.path().join("test.txt");
        let file_path_str = file_path.to_str().unwrap();
        let mut file = File::create(&file_path)?;
        write!(file, "test hash")?;
        let page_info =
            page_info_with_hash("VKZIO4rKVcnfKjW69x2ZZd39YjRo2B1RIpvV630eHBs=", "tg_id_1");

        let result = handle_new_page(file_path_str.to_string(), &page_info);
        assert_eq!(result, PageResult::TelegramId("tg_id_1".to_string()));

        let page_info_different_hash = page_info_with_hash("new_hash", "tg_id_2");
        let result = handle_new_page(file_path_str.to_string(), &page_info_different_hash);
        assert_eq!(result, PageResult::FilePath(file_path_str.to_string()));

        Ok(())
    }

    fn page_info_with_hash(hash: &str, tg_file_id: &str) -> PageInfo {
        PageInfo {
            telegram_file_id: tg_file_id.to_string(),
            file_hash: hash.to_string(),
            page_url: "page_url".to_string(),
            timestamp_ms: datetime!(2020-01-01 00:00:00),
        }
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
