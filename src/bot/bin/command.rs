use ::teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
pub(crate) enum Command {
    #[command(description = "Show all commands")]
    Help,

    #[command(description = "Get a web page by the URL")]
    GetPage { url: String }
}