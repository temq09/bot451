use std::path::PathBuf;
use std::sync::Arc;

use dptree::case;
use teloxide::dispatching::UpdateHandler;
use teloxide::types::InputFile;
use teloxide::{prelude::*, utils::command::BotCommands};

use api::{PageData, PageResult, PageWorker};
use botbackend::parallel_page_worker::ParallelPageWorker;
use proto::command::Command;

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() {
    let worker = ParallelPageWorker::new("/Users/artemushakov/prog/tmp/singlefile".to_string());
    let bot = Bot::from_env();
    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![Arc::new(worker)])
        .build()
        .dispatch()
        .await;
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    Update::filter_message()
        .filter_command::<Command>()
        .branch(case![Command::Help].endpoint(print_help))
        .branch(case![Command::GetPage { url }].endpoint(get_page))
}

async fn get_page(
    bot: Bot,
    url: String,
    message: Message,
    worker: Arc<ParallelPageWorker>,
) -> HandlerResult {
    let page_data = PageData::from_url(url, message.chat.id.to_string());
    let page_result = worker.submit_page_generation(page_data).await?;
    println!("Chat id {}", message.chat.id);
    send_document(message.chat.id.to_string(), &bot, page_result).await?;
    return Ok(());
}

async fn send_document(chat_id: String, bot: &Bot, result: PageResult) -> anyhow::Result<()> {
    if let Some(document) = result_to_input_file(result) {
        bot.send_document(chat_id, document).await?;
    }
    return Ok(());
}

fn result_to_input_file(result: PageResult) -> Option<InputFile> {
    match result {
        PageResult::FilePath(path) => Some(InputFile::file(PathBuf::from(path))),
        PageResult::Noop => None,
    }
}

async fn print_help(bot: Bot, message: Message) -> HandlerResult {
    bot.send_message(message.chat.id, Command::descriptions().to_string())
        .await
        .unwrap();
    return Ok(());
}
