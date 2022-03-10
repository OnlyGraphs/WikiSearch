use crate::{PreIndex};
use crate::{
    errors::{IndexError, IndexErrorKind},
    index::{Index},
    index_structs::{Citation, Document, Infobox},
};
use async_trait::async_trait;
use itertools::{Itertools, Chunk};
use log::{info, error};
use sqlx::Row;
use sqlx::{postgres::PgPoolOptions, query, query_scalar};
use std::collections::HashMap;
use std::future::Future;
use std::ops::Range;
use std::time::Instant;
use num_integer::Integer;

#[async_trait]
pub trait IndexBuilder  {
    async fn build_index_if_needed(&self) -> Result<Option<Index>, IndexError>;
}

pub struct SqlIndexBuilder {
    pub connection_string: String,
    pub dump_id: u32,
}

#[async_trait]
impl IndexBuilder for SqlIndexBuilder {
    async fn build_index_if_needed(&self) -> Result<Option<Index>, IndexError> 
    {
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


        let mut pre_index = PreIndex::default();
        pre_index.dump_id = highest_dump_id;

        // do this in batches 
        let nquery : i64 = query("SELECT MAX(a.articleid)
                            FROM article as a
                            ")
                            .fetch_one(&pool)
                            .await
                            .unwrap()
                            .get("max");
                        
        let num_docs = nquery as u32;
        let batch_size = 10000;
        let num_batches = num_docs.div_ceil(&batch_size);
        let mut processed_docs = 0;

        let mut timer;
        for batch in 0..num_batches {
            timer = Instant::now();
            let start_idx = batch * batch_size;
            let end_idx = ((batch + 1) * batch_size) - 1;

            let query = query("
            SELECT sub.articleid,sub.title,sub.lastupdated,sub.categories,sub.abstracts,sub.links,sub.text,sub.infoboxes, ARRAY_REMOVE(ARRAY_AGG(cit.body), NULL) as citations
            FROM (SELECT DISTINCT a.articleid,
                     a.title,
                     a.lastupdated,
                     min(c.categories) as categories,
                     min(c.abstracts) as abstracts,
                     min(c.links) as links ,
                     min(c.text) as text, 
                     ARRAY_REMOVE(ARRAY_AGG( (CASE WHEN i.infoboxtype IS NULL THEN (NULL) ELSE (i.infoboxtype,i.body) END)), NULL) as infoboxes
                    FROM article as a
                   LEFT JOIN \"content\" as c ON a.articleid = c.articleid 
                   LEFT JOIN infoboxes as i ON a.articleid = i.articleid
                   WHERE a.articleid BETWEEN $1 AND $2
                   GROUP BY a.articleid) as sub
             LEFT JOIN citations as cit ON sub.articleid = cit.articleid
             GROUP BY sub.articleid, sub.title, sub.lastupdated, sub.categories, sub.abstracts, sub.links, sub.text, sub.infoboxes
             ")
                                .bind(&start_idx)
                                .bind(&end_idx)
                                .fetch_all(&pool)
                                .await.unwrap();
                        
            query.into_iter().for_each(|v| {
                processed_docs += 1;
                let doc_id : i64 = v.get("articleid");
                let infoboxes : Option<Vec<(String,String)>> = v.get("infoboxes");
                let citations : Option<Vec<String>> = v.get("citations");
                let new_document = Box::new(Document {
                    doc_id: doc_id as u32,
                    categories: v.get("categories"),
                    main_text: v.get("text"),
                    article_links: v.get("links"),
                    title: v.get("title"),
                    last_updated_date: v.get("lastupdated"),
                    infoboxes: infoboxes.unwrap_or_default().into_iter().map(|v| Infobox {itype : v.0, text :v.1}).collect(),//v.get("infoboxes").map(|v| v).unwrap_or_default(),
                    citations: citations.unwrap_or_default().into_iter().map(|v| Citation {text :v}).collect(),//v.try_get("citations").unwrap_or_default(),
                });
                    
                if let Err(e) = pre_index.add_document(new_document){
                    let title : String = v.get("title");
                    error!("Error in adding document with {{id: {}, title: {}}}:{}",doc_id,title,e);
                }

            });

            info!("Building pre-index: {}% ({}s)",(processed_docs as f32 / num_docs as f32) * 100.0,timer.elapsed().as_secs());
        }

        pool.close().await;

        let idx = Index::from_pre_index(pre_index);

        Ok(Some(idx))
    }
}
