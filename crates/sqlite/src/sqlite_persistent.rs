use anyhow::bail;
use async_trait::async_trait;
use sqlx::sqlite::SqliteRow;
use sqlx::types::time::OffsetDateTime;
use sqlx::{Pool, Row, Sqlite, SqlitePool};

use api::{PageInfo, PagePersistent};

pub struct SqlitePagePersistent {
    connection: SqlitePool,
}

pub async fn init_db(path: String) -> anyhow::Result<SqlitePagePersistent> {
    let pool = SqlitePool::connect(&path).await?;
    create_table_if_exist(&pool).await?;
    Ok(SqlitePagePersistent { connection: pool })
}

pub async fn in_memory_db() -> anyhow::Result<SqlitePagePersistent> {
    init_db("sqlite::memory:".to_string()).await
}

const CREATE_TABLE_QUERY: &str = r#"
    CREATE TABLE IF NOT EXISTS telegram_documents (
            id INTEGER PRIMARY KEY,
            page_url TEXT NOT NULL,
            file_hash TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            telegram_file_id TEXT NOT NULL)
    "#;

const INSERT_QUERY: &str = r#"
    INSERT INTO telegram_documents (page_url, file_hash, timestamp, telegram_file_id)
    VALUES ($1, $2, $3, $4)
    "#;

async fn create_table_if_exist(connection: &Pool<Sqlite>) -> anyhow::Result<()> {
    sqlx::query(CREATE_TABLE_QUERY).execute(connection).await?;
    Ok(())
}

#[async_trait]
impl PagePersistent for SqlitePagePersistent {
    async fn save(&self, page_info: &PageInfo) -> anyhow::Result<()> {
        let count = sqlx::query(INSERT_QUERY)
            .bind(&page_info.page_url)
            .bind(&page_info.file_hash)
            .bind(page_info.timestamp_ms)
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

fn map_row(row: SqliteRow) -> anyhow::Result<Option<PageInfo>> {
    let page_info = PageInfo {
        page_url: row.try_get(1)?,
        file_hash: row.try_get(2)?,
        timestamp_ms: row.try_get::<OffsetDateTime, usize>(3)?,
        telegram_file_id: row.try_get(4)?,
    };

    return Ok(Some(page_info));
}

#[cfg(test)]
mod test {
    use sqlx::types::time::{Date, OffsetDateTime, Time};
    use time::Month;

    use api::{PageInfo, PagePersistent};

    use crate::sqlite_persistent::init_db;

    #[sqlx::test]
    async fn test_save_and_get_record() -> anyhow::Result<()> {
        let db = init_db("sqlite::memory:".to_string()).await?;
        let page_info = PageInfo {
            telegram_file_id: "telegram_file_id".to_string(),
            file_hash: "file_hash".to_string(),
            page_url: "url".to_string(),
            timestamp_ms: OffsetDateTime::new_utc(
                Date::from_calendar_date(2024, Month::January, 02)?,
                Time::from_hms(10, 10, 10)?,
            ),
        };
        db.save(&page_info).await?;
        let result = db.get("url").await?;

        assert_eq!(Some(page_info), result);

        return Ok(());
    }
}
