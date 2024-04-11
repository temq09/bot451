use clap::Parser;

#[derive(Parser)]
#[command(about, long_about = None)]
pub(crate) struct BackendArgs {
    /// Postgres SQL url, if not set Sqlite DB will be used
    #[arg(long, value_name = "URL")]
    pub(crate) postgres_url: Option<String>,

    /// Working directory where all pages will be downloaded initially
    pub(crate) work_dir: Option<String>,
}
