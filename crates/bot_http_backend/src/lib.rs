use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use api::{PageData, PageResult, PageWorker};

pub struct HttpBotBackendParams {
    backend_endpoint: String,
    client: Client,
}

impl HttpBotBackendParams {
    pub fn new(url: String) -> Self {
        HttpBotBackendParams {
            backend_endpoint: url,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl PageWorker for HttpBotBackendParams {
    async fn submit_page_generation(&self, page_data: PageData) -> anyhow::Result<PageResult> {
        let body = json!({
            "page_url": page_data.url,
            "user_id": page_data.user_id
        });
        self.client
            .post(&self.backend_endpoint)
            .json(&body)
            .send()
            .await?;
        return Ok(PageResult::Noop);
    }
}
