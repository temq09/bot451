use clap::Parser;

#[derive(Parser)]
#[command(about, long_about = None)]
pub(crate) struct BackendArgs {
    /// Postgres SQL url, if not set Sqlite DB will be used
    #[arg(long, value_name = "URL")]
    pub(crate) postgres_url: Option<String>,

    /// Working directory where all pages will be downloaded initially
    #[arg(long, value_name = "PATH")]
    pub(crate) work_dir: Option<String>,

    /// PostgresSQL user name
    #[arg(long, value_name = "USERNAME")]
    pub(crate) pg_user: Option<String>,

    /// PostgresSQL password
    #[arg(long, value_name = "PASSWORD")]
    pub(crate) pg_password: Option<String>,

    /// PostgresSQL table
    #[arg(long, value_name = "TABLE_NAME")]
    pub(crate) pg_table: Option<String>,
}
