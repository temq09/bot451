use async_trait::async_trait;

#[async_trait]
pub(crate) trait PageLoader: Sync + Send {
    async fn load_page(&self, url: String, chat_id: String);
}
