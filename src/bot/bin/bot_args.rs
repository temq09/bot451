use clap::Parser;

#[derive(Parser)]
#[command(about, long_about = None)]
pub(crate) struct BotArgs {
    /// Backend url the bot will interact with
    #[arg(long, value_name = "URL")]
    pub(crate) backend_url: Option<String>,

    /// Path to singlefile binary
    #[arg(env)]
    pub(crate) singlefile_cli: Option<String>,

    /// Path to a work dir where pages will be downloaded to. Must be set for Standalone mode
    #[arg(long, value_name = "PATH")]
    pub(crate) work_dir: Option<String>,

    /// Throttling timeout for load page request coming from the same user
    #[arg(long, value_name = "SECONDS", default_value_t = 10)]
    pub(crate) throttling_timeout_seconds: u64,
}
