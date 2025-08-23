use async_trait::async_trait;
use teloxide::prelude::Requester;
use teloxide::types::{FileId, InputFile};
use teloxide::Bot;

use api::{PageResult, PageUploader};

pub(crate) struct TeloxidePageUploader {
    bot: Bot,
}

impl TeloxidePageUploader {
    pub(crate) fn new_from_env() -> Self {
        TeloxidePageUploader {
            bot: Bot::from_env(),
        }
    }
}

#[async_trait]
impl PageUploader for TeloxidePageUploader {
    async fn send_page(
        &self,
        chat_id: &str,
        page_result: &PageResult,
    ) -> anyhow::Result<Option<String>> {
        println!("Sending page to {}", chat_id);
        let input_file = to_input_file(page_result);
        let result = self
            .bot
            .send_document(chat_id.to_string(), input_file)
            .await?
            .document()
            .map(|document| document.file.id.to_string());
        return Ok(result);
    }
}

fn to_input_file(page_result: &PageResult) -> InputFile {
    match page_result {
        PageResult::FilePath(path) => InputFile::file(path),
        PageResult::TelegramId(id) => InputFile::file_id(FileId::from(id.to_string())),
    }
}
