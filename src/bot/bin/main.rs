use std::path::PathBuf;
use std::sync::Arc;

use dptree::case;
use teloxide::{prelude::*, utils::command::BotCommands};
use teloxide::dispatching::UpdateHandler;
use teloxide::types::InputFile;

use botbackend::{PageData, PageResult, PageWorker};
use botbackend::parallel_page_worker::ParallelPageWorker;

use crate::command::Command;

mod command;

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() {
    let bot = Bot::new("7085983112:AAGpitXfAqPwvpu_czrP9kiMOUlg4iIwJoo");
    let worker = ParallelPageWorker::new("/Users/artemushakov/prog/tmp/singlefile".to_string());
    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![Arc::new(worker)])
        .build()
        .dispatch()
        .await;
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    Update::filter_message().filter_command::<Command>()
        .branch(case![Command::Help].endpoint(print_help))
        .branch(case![Command::GetPage { url }].endpoint(get_page))
}

async fn get_page(bot: Bot, url: String, message: Message, worker: Arc<ParallelPageWorker>) -> HandlerResult {
    let page_result = worker.submit_page_generation(PageData::from_url(url)).await?;
    send_document(message.chat.id.to_string(), &bot, page_result).await?;
    return Ok(());
}

async fn send_document(chat_id: String, bot: &Bot, result: PageResult) -> anyhow::Result<()> {
    bot.send_document(chat_id, result_to_input_file(result)).await?;
    return Ok(());
}

fn result_to_input_file(result: PageResult) -> InputFile {
    match result { PageResult::FilePath(path) => InputFile::file(PathBuf::from(path)) }
}

async fn print_help(bot: Bot, message: Message) -> HandlerResult {
    bot.send_message(message.chat.id, Command::descriptions().to_string()).await.unwrap();
    return Ok(());
}
