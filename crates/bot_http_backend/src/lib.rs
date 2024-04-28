use async_trait::async_trait;
use reqwest::{Client, Url};
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
            // "user_id": page_data.user_id
        });
        let mut url = Url::parse(&self.backend_endpoint)?;
        url.set_path("v1/requestPageForUser");
        self.client.post(url).json(&body).send().await?;
        todo!()
        // return Ok(PageResult::Noop);
    }
}
