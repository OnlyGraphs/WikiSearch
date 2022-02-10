use crate::index::index::{BasicIndex, IndexInterface};
use crate::index::index_structs::Document;
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
            Err(e) => Err(format!("Current dumpid is gibberish. {e}")),
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
            //TODO!: change these fields as appropriate
            let new_document = Document {
                doc_id: row.articleid as u32,
                title: "".to_string(),
                categories: row.categories.unwrap_or("".to_string()),
                last_updated_date: "".to_string(),
                namespace: 0,
                article_abstract: row.abstracts.unwrap_or("".to_string()),
                infobox_text: Vec::new(),
                infobox_type: "Fruit".to_string(),
                infobox_ids: Vec::new(),
                main_text: row.text.unwrap_or("".to_string()),
                article_links: row.links.unwrap_or("".to_string()),
                citations_text: Vec::new(),
                citations_ids: Vec::new(),
            };
            idx.add_document(new_document);
        }

        pool.close().await;
        Ok(idx)
    }
}
