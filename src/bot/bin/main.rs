use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Context};
use clap::Parser;
use dptree::case;
use teloxide::dispatching::UpdateHandler;
use teloxide::{prelude::*, utils::command::BotCommands};

use proto::command::Command;

use crate::bot_args::{BotArgs, Mode};
use crate::bot_error::BotError;
use crate::worker::page_loader::PageLoader;
use crate::worker::remote_page_loader::RemotePageLoader;
use crate::worker::standalone_page_loader::StandalonePageLoader;
use crate::worker::throttled_page_loader::ThrottlePageLoader;

mod bot_args;
mod bot_error;
mod worker;

type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bot = Bot::from_env();
    let args = BotArgs::parse();
    let duration = Duration::from_secs(args.throttling_timeout_seconds.clone());
    let worker = create_worker(args, bot.clone())?;
    let throttle_worker: Arc<dyn PageLoader> = Arc::new(ThrottlePageLoader::new(duration, worker));
    Dispatcher::builder(bot, schema())
        .dependencies(dptree::deps![throttle_worker])
        .build()
        .dispatch()
        .await;

    Ok(())
}

fn schema() -> UpdateHandler<Box<dyn std::error::Error + Send + Sync + 'static>> {
    Update::filter_message()
        .filter_command::<Command>()
        .branch(case![Command::Help].endpoint(print_help))
        .branch(case![Command::GetPage { url }].endpoint(get_page))
}

async fn get_page(
    url: String,
    message: Message,
    worker: Arc<dyn PageLoader>,
    bot: Bot,
) -> HandlerResult {
    println!("Chat id {}", message.chat.id);
    let result = worker
        .load_page(url.to_string(), message.chat.id.to_string())
        .await;
    match result {
        Ok(_) => {}
        Err(e) => handle_error(bot, message, e).await?,
    };

    return Ok(());
}

async fn print_help(bot: Bot, message: Message) -> HandlerResult {
    bot.send_message(message.chat.id, Command::descriptions().to_string())
        .await?;
    return Ok(());
}

async fn handle_error(bot: Bot, message: Message, bot_error: BotError) -> HandlerResult {
    match bot_error {
        BotError::ThrottleError => {
            send_message(
                bot,
                message.chat.id,
                "Too many requests. Try again later".to_string(),
            )
            .await
        }
        _ => {
            println!("Error during page loading: {:?}", bot_error);
            Ok(())
        }
    }
}

async fn send_message(bot: Bot, chat_id: ChatId, message: String) -> HandlerResult {
    let _ = bot.send_message(chat_id, message).await?;
    Ok(())
}

fn create_worker(args: BotArgs, bot: Bot) -> anyhow::Result<Box<dyn PageLoader>> {
    match args.mode.unwrap_or(Mode::Standalone) {
        Mode::Standalone => {
            let singlefile_cli_path = args
                .singlefile_cli
                .context("SINGLEFILE_CLI env variable must be set for the standalone mode")?;

            let work_dir = args
                .work_dir
                .context("Working dir path must be set for standalone mode")?;
            Ok(Box::new(StandalonePageLoader::new(
                singlefile_cli_path,
                work_dir,
                bot,
            )))
        }
        Mode::Distributed => {
            let backend_url = args
                .backend_url
                .ok_or_else(|| anyhow!("Backend urls must be set for the distributed variant"))?;
            let loader = RemotePageLoader::new(backend_url.as_str())?;
            Ok(Box::new(loader))
        }
    }
}
