use anyhow::bail;
use async_trait::async_trait;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions, PgRow};
use sqlx::types::time::OffsetDateTime;
use sqlx::{PgPool, Row};

use api::{PageInfo, PagePersistent};

pub struct PostgresPersistent {
    connection: PgPool,
}

impl PostgresPersistent {
    async fn connect(
        user_name: &str,
        password: &str,
        database: &str,
        host: &str,
    ) -> anyhow::Result<PostgresPersistent> {
        let options = PgConnectOptions::new()
            .host(host)
            .database(database)
            .password(password)
            .username(user_name);

        let pool = PgPoolOptions::new().connect_with(options).await?;

        init_database(&pool).await?;

        Ok(PostgresPersistent { connection: pool })
    }
}

async fn init_database(connection: &PgPool) -> anyhow::Result<()> {
    sqlx::query(
        r#"
            CREATE TABLE IF NOT EXISTS telegram_documents (
                id INTEGER PRIMARY KEY,
                page_url TEXT NOT NULL,
                file_hash TEXT NOT NULL,
                timestamp TIMESTAMP NOT NULL,
                telegram_file_id TEXT NOT NULL)
    "#,
    )
    .execute(connection)
    .await?;
    Ok(())
}

#[async_trait]
impl PagePersistent for PostgresPersistent {
    async fn save(&self, page_info: &PageInfo) -> anyhow::Result<()> {
        let count = sqlx::query(
            r#"
                INSERT INTO telegram_documents (page_url, file_hash, timestamp, telegram_file_id)
                VALUES ($1, $2, $3, $4)
                "#,
        )
        .bind(&page_info.page_url)
        .bind(&page_info.file_hash)
        .bind(page_info.timestamp_ms.to_string().as_str())
        .bind(&page_info.telegram_file_id)
        .execute(&self.connection)
        .await?
        .rows_affected();

        return if count == 1 {
            Ok(())
        } else {
            bail!("Expected one row to be inserted, but was {}", count)
        };
    }

    async fn get(&self, page_url: &str) -> anyhow::Result<Option<PageInfo>> {
        let result = sqlx::query(
            r#"
            SELECT * FROM telegram_documents
            WHERE page_url = $1
            "#,
        )
        .bind(page_url)
        .fetch_optional(&self.connection)
        .await?;

        return match result {
            None => Ok(None),
            Some(row) => map_row(row),
        };
    }
}

fn map_row(row: PgRow) -> anyhow::Result<Option<PageInfo>> {
    let page_info = PageInfo {
        page_url: row.try_get(1)?,
        file_hash: row.try_get(2)?,
        timestamp_ms: row.try_get::<OffsetDateTime, usize>(3)?,
        telegram_file_id: row.try_get(4)?,
    };

    return Ok(Some(page_info));
}
