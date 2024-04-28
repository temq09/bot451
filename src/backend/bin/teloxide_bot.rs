use async_trait::async_trait;
use teloxide::prelude::Requester;
use teloxide::types::InputFile;
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
        let result = match to_input_file(page_result) {
            None => None,
            Some(input_file) => self
                .bot
                .send_document(chat_id, input_file)
                .await?
                .document()
                .map(|document| document.file.id.to_string()),
        };
        return Ok(result);
    }
}

fn to_input_file(page_result: &PageResult) -> Option<InputFile> {
    match page_result {
        PageResult::FilePath(path) => Some(InputFile::file(path)),
        PageResult::TelegramId(id) => Some(InputFile::file_id(id)),
    }
}
