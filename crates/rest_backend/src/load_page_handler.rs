use api::{PageData, PageUploader, PageWorker};

pub struct LoadPageHandler {
    page_loader: Box<dyn PageWorker>,
    page_uploader: Box<dyn PageUploader>,
}

impl LoadPageHandler {
    pub(crate) fn new(loader: Box<dyn PageWorker>, page_uploader: Box<dyn PageUploader>) -> Self {
        LoadPageHandler {
            page_loader: loader,
            page_uploader,
        }
    }

    pub(crate) async fn load_page_for_user(
        &self,
        page_url: String,
        chat_id: String,
    ) -> anyhow::Result<()> {
        let result = self
            .page_loader
            .submit_page_generation(PageData::from_url(page_url, chat_id.clone()))
            .await?;
        self.page_uploader.send_page(chat_id, result).await?;
        return Ok(());
    }
}
