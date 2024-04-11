use clap::{Parser, ValueEnum};

#[derive(Parser)]
pub(crate) struct BotArgs {
    #[arg(value_enum)]
    pub(crate) mode: Option<Mode>,
    #[arg(long)]
    pub(crate) backend_url: Option<String>,
}

#[derive(ValueEnum, Copy, Clone)]
pub(crate) enum Mode {
    Standalone,
    Distributed,
}
