use async_trait::async_trait;
use reqwest::{Client, Url};
use serde_json::json;

use crate::bot_error::BotError;
use crate::worker::page_loader::PageLoader;

pub(crate) struct RemotePageLoader {
    backend_url: Url,
    client: Client,
}

impl RemotePageLoader {
    pub(crate) fn new(backend_url: &str) -> anyhow::Result<Self> {
        let url = Url::parse(backend_url)?;
        Ok(RemotePageLoader {
            backend_url: url,
            client: Client::new(),
        })
    }
}

#[async_trait]
impl PageLoader for RemotePageLoader {
    async fn load_page(&self, url: String, chat_id: String) -> Result<(), BotError> {
        let body = json!({
            "page_url": url,
            "user_id": chat_id
        });
        let mut request_page_url = self.backend_url.clone();
        request_page_url.set_path("v1/requestPageForUser");
        self.client
            .post(request_page_url)
            .json(&body)
            .send()
            .await?;
        Ok(())
    }
}
