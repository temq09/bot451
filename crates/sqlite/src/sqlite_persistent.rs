use rusqlite::{Connection, Row};

use api::{PageInfo, PagePersistent};

pub struct SqlitePagePersistent {
    connection: Connection,
}

fn init_db(path: String) -> anyhow::Result<SqlitePagePersistent> {
    let connection = Connection::open(path)?;
    create_table_if_exist(&connection)?;
    Ok(SqlitePagePersistent { connection })
}

const COLUMN_ID: &str = "id";
const COLUMN_PAGE_URL: &str = "page_url";
const COLUMN_FILE_HASH: &str = "file_hash";
const COLUMN_ID_TIMESTAMP: &str = "timestamp";
const COLUMN_TELEGRAM_FILE_ID: &str = "telegram_file_id";
const CREATE_TABLE_QUERY: &str = format!(
    "CREATE TABLE IF NOT EXISTS telegram_documents (
            {COLUMN_ID} INTEGER PRIMARY KEY,
            {COLUMN_PAGE_URL} TEXT NOT NULL
            {COLUMN_FILE_HASH} TEXT NOT NULL
            {COLUMN_ID_TIMESTAMP} INTEGER NOT NULL
            {COLUMN_TELEGRAM_FILE_ID} TEXT NOT NULL
        )"
);

const INSERT_QUERY: &str = format!(
    "INSERT INTO telegram_documents \
    ({COLUMN_PAGE_URL}, {COLUMN_FILE_HASH}, {COLUMN_ID_TIMESTAMP}, {COLUMN_TELEGRAM_FILE_ID}) \
    VALUES (:{COLUMN_PAGE_URL}), (:{COLUMN_FILE_HASH}), (:{COLUMN_ID_TIMESTAMP}), {COLUMN_TELEGRAM_FILE_ID}"
);

fn create_table_if_exist(connection: &Connection) -> anyhow::Result<()> {
    connection.execute(CREATE_TABLE_QUERY, ())?;
    return Ok(());
}

impl PagePersistent for SqlitePagePersistent {
    async fn save(&self, page_info: &PageInfo) -> anyhow::Result<()> {
        self.connection.execute(
            INSERT_QUERY,
            &[
                (format!(":{}", COLUMN_PAGE_URL), page_info.page_url.as_str()),
                (
                    format!(":{}", COLUMN_FILE_HASH),
                    page_info.file_hash.as_str(),
                ),
                (
                    format!(":{}", COLUMN_ID_TIMESTAMP),
                    page_info.timestamp_ms.to_string().as_str(),
                ),
                (
                    format!(":{}", COLUMN_TELEGRAM_FILE_ID),
                    page_info.telegram_file_id.as_str(),
                ),
            ],
        )?;

        return Ok(());
    }

    async fn get(&self, page_url: &str) -> anyhow::Result<Option<PageInfo>> {
        Ok(self
            .connection
            .query_row(
                "SELECT * FROM telegram_documents WHERE page_url",
                [page_url],
                |row| map_row(row),
            )
            .map(|page_info| Some(page_info))
            .map_err(|err| None)?)
    }
}

fn map_row(row: &Row) -> rusqlite::Result<PageInfo> {
    Ok(PageInfo {
        page_url: row.get(1)?,
        file_hash: row.get(2)?,
        timestamp_ms: row.get(3).map(|timestamp| u128::from(timestamp))?,
        telegram_file_id: row.get(4)?,
    })
}

#[cfg(test)]
mod test {
    use api::{PageInfo, PagePersistent};

    use crate::sqlite_persistent::init_db;

    #[tokio::test]
    async fn test_save_and_get_record() -> anyhow::Result<()> {
        let db = init_db(":memory".to_string())?;
        let page_info = PageInfo {
            telegram_file_id: "telegram_file_id".to_string(),
            file_hash: "file_hash".to_string(),
            page_url: "url".to_string(),
            timestamp_ms: 123,
        };
        db.save(&page_info).await?;
        let result = db.get("url").await?;

        assert_eq!(Some(page_info), result);

        return Ok(());
    }
}
