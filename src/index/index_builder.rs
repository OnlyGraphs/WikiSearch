use crate::index::index::BasicIndex;
use async_trait::async_trait;
use sqlx::{postgres::PgPoolOptions, query};
use std::fs;

#[derive(Debug)]
pub enum BuildErrorCode {
    Access(String),
    Permissions(String),
    MissingData(String),
    Server(String),
    Teapot(String),
}

#[async_trait]
pub trait IndexBuilder {
    async fn build_index(&self) -> Result<BasicIndex, BuildErrorCode>;
}

pub struct SqlIndexBuilder {
    pub connection_string: String,
    pub dump_id: i32,
}

impl SqlIndexBuilder {
    pub fn get_dump_id() -> Result<i32, String> {
        return match fs::read_to_string("dumpid")
            .unwrap_or("-1".to_string())
            .parse()
        {
            Ok(v) => Ok(v),
            Err(E) => Err("Current dumpid is gibberish.".to_string()),
        };
    }
}

#[async_trait]
impl IndexBuilder for SqlIndexBuilder {
    async fn build_index(&self) -> Result<BasicIndex, BuildErrorCode> {
        let pool = match PgPoolOptions::new()
            .max_connections(1)
            .connect(&self.connection_string)
            .await
        {
            Ok(pool) => pool,
            Err(error) => return Err(BuildErrorCode::Access(error.to_string())),
        };

        let res = match query!(
            "SELECT * FROM content
                                ORDER BY articleid ASC "
        )
        .fetch_all(&pool)
        .await
        {
            Ok(res) => res,
            Err(error) => return Err(BuildErrorCode::Server(error.to_string())),
        };

        let mut idx = BasicIndex::default();

        for row in res {
            idx.add_document(
                &row.text.unwrap_or("".to_string()),
                row.articleid as u32,
                &row.categories.unwrap_or("".to_string()),
                &row.links.unwrap_or("".to_string()),
                &row.abstracts.unwrap_or("".to_string()),
            );
        }

        pool.close().await;
        Ok(idx)
    }
}
