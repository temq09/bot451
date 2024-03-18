// use std::fs::File;
//
// use teloxide::Bot;
// use teloxide::prelude::{Message, Requester};
// use teloxide::repls::CommandReplExt;
// use teloxide::utils::command::BotCommands;
//
// use botbackend::{PageData, PageWorker};
//
// use crate::command::Command;
//
// pub(crate) struct StandaloneBot {
//     telegram_bot: Bot,
//     worker: dyn PageWorker,
// }
//
// impl StandaloneBot {
//     pub(crate) fn new(bot: Bot, worker: Box<dyn PageWorker>) -> Box<Self> {
//         Box::new(
//             StandaloneBot {
//                 telegram_bot: bot,
//                 worker,
//             }
//         )
//     }
// }
//
// impl StandaloneBot {
//     pub async fn run(self) {
//         Command::repl(self.telegram_bot, handle_message).await;
//     }
//
//
//
// }
//
// async fn handle_message(bot: Bot,
//                         msg: Message,
//                         command: Command,
// ) -> anyhow::Result<()> {
//     match command {
//         Command::GetPage { url } =>
//             get_page(url, worker).await?.and_then(|file| send_document(msg.chat.id, bot, file)),
//         Command::Help => print_help(bot, msg.chat.id.to_string()).await,
//     }?;
//
//     Ok(())
// }
//
// async fn get_page(url: String, worker: &dyn PageWorker) -> anyhow::Result<File> {
//     worker.submit_page_generation(PageData::from_url(url))
// }
//
// async fn send_document(chat_id: String, bot: &Bot, file: File) -> anyhow::Result<()> {
//     // bot.send_document(chat_id, InputFile::file()).await?;
//     return Ok(());
// }
//
// async fn print_help(bot: Bot, chat_id: String) {
//     bot.send_message(chat_id, Command::descriptions().to_string()).await.unwrap();
// }