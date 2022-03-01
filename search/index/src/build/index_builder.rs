
use crate::{PreIndex};
use crate::{
    errors::{IndexError, IndexErrorKind},
    index::{BasicIndex, Index},
    index_structs::{Citation, Document, Infobox},
};
use async_trait::async_trait;
use sqlx::{postgres::PgPoolOptions, query, query_scalar};
use std::collections::HashMap;

#[async_trait]
pub trait IndexBuilder {
    async fn build_index_if_needed(&self) -> Result<Option<Box<dyn Index>>, IndexError>;

}

pub struct SqlIndexBuilder {
    pub connection_string: String,
    pub dump_id: u32,
}

#[async_trait]
impl IndexBuilder for SqlIndexBuilder {
    async fn build_index_if_needed(&self) -> Result<Option<Box<dyn Index>>, IndexError> {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&self.connection_string)
            .await?;

        let highest_dump_id = query_scalar!(
            "SELECT MAX(article.dumpid)
             FROM article"
        )
        .fetch_one(&pool)
        .await?
        .unwrap_or(0) as u32;

        if highest_dump_id <= self.dump_id {
            return Ok(None);
        }

        let main_query = query!(
            "SELECT a.articleid, a.title, a.domain, a.namespace, a.lastupdated,
                    c.categories, c.abstracts, c.links, c.text
             From article as a, \"content\" as c
             where a.articleid = c.articleid"
        )
        .fetch_all(&pool)
        .await?;

        let infoboxes_query = query!(
            "SELECT i.articleid, i.infoboxtype, i.body
             From infoboxes as i"
        )
        .fetch_all(&pool)
        .await?;

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

        let citations_query = query!(
            "SELECT c.articleid, c.citationid, c.body
             From citations as c"
        )
        .fetch_all(&pool)
        .await?;

        let mut article_citations: HashMap<u32, Vec<Citation>> =
            HashMap::with_capacity(main_query.len());
        for i in citations_query {
            article_citations
                .entry(i.articleid as u32)
                .or_insert(Vec::new())
                .push(Citation { text: i.body })
        }

        // let mut idx =
        //     BasicIndex::<HashMap<String, PostingNode>>::with_capacity(main_query.len(), 1024, 512);
        let mut pre_index = PreIndex::default();

        pre_index.dump_id = highest_dump_id;

        for row in main_query {
            let doc_id = row.articleid.ok_or(IndexError {
                msg: "Missing articleid when querying articles".to_string(),
                kind: IndexErrorKind::Database,
            })? as u32;

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
            pre_index.add_document(new_document)?;
        }

        pool.close().await;

        let idx = BasicIndex::from_pre_index(pre_index);

        Ok(Some(idx))
    }
}
