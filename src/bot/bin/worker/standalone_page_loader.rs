use std::path::PathBuf;

use async_trait::async_trait;
use teloxide::prelude::Requester;
use teloxide::types::InputFile;
use teloxide::Bot;

use api::{PageData, PageResult, PageWorker};
use botbackend::parallel_page_worker::ParallelPageWorker;

use crate::worker::page_loader::PageLoader;

pub(crate) struct StandalonePageLoader {
    worker: ParallelPageWorker,
    bot: Bot,
}

impl StandalonePageLoader {
    pub(crate) fn new(singlefile_cli_path: String, work_dir: String, bot: Bot) -> Self {
        let worker = ParallelPageWorker::new(work_dir, singlefile_cli_path);
        StandalonePageLoader { worker, bot }
    }
}

#[async_trait]
impl PageLoader for StandalonePageLoader {
    async fn load_page(&self, url: String, chat_id: String) {
        let page_data = PageData::from_url(url);
        let result = self.worker.submit_page_generation(page_data).await;
        if let Ok(page_result) = result {
            send_document(chat_id, &self.bot, page_result).await
        }
    }
}

async fn send_document(chat_id: String, bot: &Bot, result: PageResult) {
    if let Some(document) = result_to_input_file(result) {
        let _ = bot.send_document(chat_id, document).await;
    }
}

fn result_to_input_file(result: PageResult) -> Option<InputFile> {
    match result {
        PageResult::FilePath(path) => Some(InputFile::file(PathBuf::from(path))),
        PageResult::TelegramId(id) => Some(InputFile::file_id(id)),
    }
}
