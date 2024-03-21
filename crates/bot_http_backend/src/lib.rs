use api::{PageData, PageResult, PageWorker};

pub struct HttpBotBackendParams {
    backend_endpoint: String,
}

impl HttpBotBackendParams {
    pub fn new(url: String) -> Self {
        HttpBotBackendParams {
            backend_endpoint: url,
        }
    }
}

impl PageWorker for HttpBotBackendParams {
    async fn submit_page_generation(&self, page_data: PageData) -> anyhow::Result<PageResult> {
        todo!()
    }
}