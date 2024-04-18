use std::io::Read;
use std::sync::Arc;

use time::{OffsetDateTime, PrimitiveDateTime};

use api::{PageData, PageInfo, PagePersistent, PageResult, PageUploader, PageWorker};
use utils::hash::make_hash_for_file;

pub struct LoadPageHandler {
    page_loader: Box<dyn PageWorker>,
    page_uploader: Box<dyn PageUploader>,
    cache: Arc<dyn PagePersistent>,
}

impl LoadPageHandler {
    pub(crate) fn new(
        loader: Box<dyn PageWorker>,
        page_uploader: Box<dyn PageUploader>,
        cache: Arc<dyn PagePersistent + 'static>,
    ) -> Self {
        LoadPageHandler {
            page_loader: loader,
            page_uploader,
            cache,
        }
    }

    pub(crate) async fn load_page_for_user(
        &self,
        page_url: String,
        chat_id: String,
    ) -> anyhow::Result<()> {
        let result = self
            .page_loader
            .submit_page_generation(PageData::from_url(page_url.clone(), chat_id.clone()))
            .await?;
        let file_id = self.page_uploader.send_page(chat_id, &result).await?;
        if let Some(file_id) = file_id {
            save_to_cache(&file_id, &result, &self.cache, page_url).await;
        }
        return Ok(());
    }
}

async fn save_to_cache(
    file_id: &str,
    result: &PageResult,
    cache: &Arc<dyn PagePersistent>,
    page_url: String,
) {
    let current_time = OffsetDateTime::now_utc();
    let primitive_time = PrimitiveDateTime::new(current_time.date(), current_time.time());
    let page_info = prepare_page_hash(&result).map(|hash| PageInfo {
        telegram_file_id: file_id.to_string(),
        file_hash: hash,
        page_url,
        timestamp_ms: primitive_time,
    });

    if let Some(page_info) = page_info {
        let _ = cache.save(&page_info).await;
    }
}

fn prepare_page_hash(page_result: &PageResult) -> Option<String> {
    match page_result {
        PageResult::FilePath(path) => make_hash_for_file(path),
        PageResult::TelegramId(_) | PageResult::Noop => None,
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::Write;

    use tempfile::tempdir;

    use api::PageResult;

    use crate::load_page_handler::prepare_page_hash;

    #[test]
    fn test_prepare_page_info_empty_result() {
        assert_eq!(prepare_page_hash(&PageResult::Noop), None);
        assert_eq!(
            prepare_page_hash(&PageResult::TelegramId("id".to_string())),
            None
        )
    }

    #[test]
    fn test_prepare_page_info_file() -> anyhow::Result<()> {
        let tmpdir = tempdir()?;
        let file_path = tmpdir.path().join("test.txt");
        let mut file = File::create(&file_path)?;
        write!(file, "test hash")?;

        let result = prepare_page_hash(&PageResult::FilePath(
            file_path.to_str().unwrap().to_string(),
        ));
        assert_eq!(
            result,
            Some("VKZIO4rKVcnfKjW69x2ZZd39YjRo2B1RIpvV630eHBs=".to_string())
        );

        Ok(())
    }
}
