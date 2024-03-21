use reqwest::Client;
use serde::Serialize;

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

impl PageWorker for HttpBotBackendParams {
    async fn submit_page_generation(&self, page_data: PageData) -> anyhow::Result<PageResult> {
        self.client.post(&self.backend_endpoint)
            .json(&SubmitPageForUserBody::from_page_data(&page_data))
            .send()
            .await?;
        return Ok(PageResult::Noop);
    }
}

#[derive(Debug, Serialize)]
struct SubmitPageForUserBody {
    page_url: String,
    user_id: String,
}

impl SubmitPageForUserBody {
    fn from_page_data(page_data: &PageData) -> Self {
        SubmitPageForUserBody {
            page_url: page_data.url.to_owned(),
            user_id: page_data.user_id.to_owned(),
        }
    }
}
