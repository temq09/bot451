use api::PageWorker;

pub struct LoadPageHandler {
    page_loader: Box<dyn PageWorker>,
}

impl LoadPageHandler {
    pub(crate) fn new(loader: Box<dyn PageWorker>) -> Self {
        LoadPageHandler {
            page_loader: loader,
        }
    }

    pub(crate) async fn load_page_for_user(
        &self,
        page_url: String,
        user_id: String,
    ) -> anyhow::Result<()> {
        return Ok(());
    }
}
