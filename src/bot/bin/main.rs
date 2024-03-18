use std::fs::File;

use teloxide::Bot;
use teloxide::dispatching::UpdateHandler;
use teloxide::dptree::case;
use teloxide::prelude::{Dispatcher, Message, Requester};
use teloxide::utils::command::BotCommands;

use botbackend::{PageData, PageWorker};
use botbackend::parallel_page_worker::ParallelPageWorker;

use crate::command::Command;

mod command;
mod standalone_bot;

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() {
    // let standalone_bot = StandaloneBot::new(
    //     Bot::new("7085983112:AAGpitXfAqPwvpu_czrP9kiMOUlg4iIwJoo"),
    //     Box::new(ParallelPageWorker::new()),
    // );
    // standalone_bot.run().await;

    let bot = Bot::new("7085983112:AAGpitXfAqPwvpu_czrP9kiMOUlg4iIwJoo");
    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![Box::new(ParallelPageWorker::new())])
        .build()
        .dispatch()
        .await;

    // let bot = Bot::new("7085983112:AAGpitXfAqPwvpu_czrP9kiMOUlg4iIwJoo");
    // Command::repl(bot, handle_message).await;
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    teloxide::filter_command::<Command, _>()
        .branch(case![Command::Help].endpoint(print_help))
        .branch(case![Command::GetPage].endpoint(get_page))
}

// async fn handle_message(bot: Bot,
//                         msg: Message,
//                         command: Command,
//                         worker: Box<dyn PageWorker>,
// ) -> HandlerResult {
//     match command {
//         Command::GetPage { url } =>
//             get_page(url, worker.as_ref()).await?.and_then(|file| send_document(msg.chat.id, bot, file)),
//         Command::Help => print_help(bot, msg.chat.id.to_string()).await,
//     }?;
//
//     Ok(())
// }

async fn get_page(url: String, worker: &dyn PageWorker) -> HandlerResult {
    worker.submit_page_generation(PageData::from_url(url))?;
    return Ok(());
}

async fn send_document(chat_id: String, bot: &Bot, file: File) -> HandlerResult {
    // bot.send_document(chat_id, InputFile::file()).await?;
    return Ok(());
}

async fn print_help(bot: Bot, message: Message) -> HandlerResult {
    bot.send_message(message.chat.id, Command::descriptions().to_string()).await.unwrap();
    return Ok(());
}
