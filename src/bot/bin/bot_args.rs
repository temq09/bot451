use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(about, long_about = None)]
pub(crate) struct BotArgs {
    /// Mode for the bot to run.
    #[arg(long, value_enum, value_name = "MODE")]
    pub(crate) mode: Option<Mode>,

    /// Backend url the bot will interact with
    #[arg(long, value_name = "URL")]
    pub(crate) backend_url: Option<String>,

    /// Path to singlefile binary
    #[arg(env)]
    pub(crate) singlefile_cli: Option<String>,

    /// Path to a work dir where pages will be downloaded to. Must be set for Standalone mode
    #[arg(long, value_name = "PATH")]
    pub(crate) work_dir: Option<String>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub(crate) enum Mode {
    /// Everything will work out of the box, no extra settings required.
    /// Use it for simple installations
    Standalone,
    /// Thin bot mode that requires the separate backend for the tasks to be executed.
    /// Use it to scale the backend workload
    Distributed,
}
