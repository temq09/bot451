use std::path::PathBuf;

use async_trait::async_trait;
use teloxide::prelude::Requester;
use teloxide::types::InputFile;
use teloxide::Bot;

use api::{PageData, PageResult, PageWorker};
use botbackend::parallel_page_worker::ParallelPageWorker;

use crate::bot_error::BotError;
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
    async fn load_page(&self, url: String, chat_id: String) -> Result<(), BotError> {
        let page_data = PageData::from_url(url);
        let result = self.worker.submit_page_generation(page_data).await?;
        send_document(chat_id, &self.bot, result).await
    }
}

async fn send_document(chat_id: String, bot: &Bot, result: PageResult) -> Result<(), BotError> {
    let document = result_to_input_file(result);
    bot.send_document(chat_id, document).await?;
    Ok(())
}

fn result_to_input_file(result: PageResult) -> InputFile {
    match result {
        PageResult::FilePath(path) => InputFile::file(PathBuf::from(path)),
        PageResult::TelegramId(id) => InputFile::file_id(id),
    }
}
