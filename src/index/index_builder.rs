use crate::index::index_structs::Citation;
use crate::index::index_structs::Infobox;
use crate::index::{
    index::{BasicIndex, Index},
    index_structs::Document,
};
use async_trait::async_trait;
use sqlx::{postgres::PgPoolOptions, query};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
pub enum BuildErrorCode {
    Access(String),
    // Permissions(String),
    // MissingData(String),
    Server(String),
    // Teapot(String),
}

#[async_trait]
pub trait IndexBuilder {
    async fn build_index(&self) -> Result<Box<dyn Index>, BuildErrorCode>;
}

pub struct SqlIndexBuilder {
    pub connection_string: String,
    pub dump_id: u32,
}

#[async_trait]
impl IndexBuilder for SqlIndexBuilder {
    async fn build_index(&self) -> Result<Box<dyn Index>, BuildErrorCode> {
        let pool = match PgPoolOptions::new()
            .max_connections(1)
            .connect(&self.connection_string)
            .await
        {
            Ok(pool) => pool,
            Err(error) => return Err(BuildErrorCode::Access(error.to_string())),
        };

        // TODO: check dump id before updating

        let main_query = match query!(
            "SELECT a.articleid, a.title, a.domain, a.namespace, a.lastupdated,
                    c.categories, c.abstracts, c.links, c.text
             From article as a, \"content\" as c
             where a.articleid = c.articleid"
        )
        .fetch_all(&pool)
        .await
        {
            Ok(main_query) => main_query,
            Err(error) => return Err(BuildErrorCode::Server(error.to_string())),
        };

        let infoboxes_query = match query!(
            "SELECT i.articleid, i.infoboxtype, i.body
             From infoboxes as i"
        )
        .fetch_all(&pool)
        .await
        {
            Ok(infoboxes_query) => infoboxes_query,
            Err(error) => return Err(BuildErrorCode::Server(error.to_string())),
        };

        let mut article_infoboxes: HashMap<u32, Vec<Infobox>> = HashMap::new();
        for i in infoboxes_query {
            article_infoboxes
                .entry(i.articleid as u32)
                .or_insert(Vec::new())
                .push(Infobox {
                    itype: i.infoboxtype,
                    text: i.body,
                })
        }

        let citations_query = match query!(
            "SELECT c.articleid, c.citationid, c.body
             From citations as c"
        )
        .fetch_all(&pool)
        .await
        {
            Ok(citations_query) => citations_query,
            Err(error) => return Err(BuildErrorCode::Server(error.to_string())),
        };

        let mut article_citations: HashMap<u32, Vec<Citation>> = HashMap::new();
        for i in citations_query {
            article_citations
                .entry(i.articleid as u32)
                .or_insert(Vec::new())
                .push(Citation { text: i.body })
        }

        let mut idx = BasicIndex::default();

        idx.set_dump_id(self.dump_id);

        for row in main_query {
            let doc_id = row.articleid.unwrap() as u32;
            let new_document = Box::new(Document {
                doc_id: doc_id,
                categories: row.categories.unwrap_or_default(),
                main_text: row.text.unwrap_or_default(),
                article_links: row.links.unwrap_or_default(),
                title: row.title.unwrap_or_default(),
                namespace: row.namespace.unwrap_or_default(),
                last_updated_date: row.lastupdated.unwrap_or_default(),
                infoboxes: article_infoboxes.remove(&doc_id).unwrap_or_default(),
                citations: article_citations.remove(&doc_id).unwrap_or_default(),
            });
            idx.add_document(new_document);
        }

        pool.close().await;
        idx.finalize();

        Ok(Box::new(idx))
    }
}
