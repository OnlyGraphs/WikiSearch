use crate::{PreIndex};
use crate::{
    errors::{IndexError},
    index::{Index},
    index_structs::{Citation, Document, Infobox},
};
use async_trait::async_trait;

use itertools::izip;
use log::{info, error};
use sqlx::Row;
use sqlx::{postgres::PgPoolOptions, query, query_scalar};



use std::collections::HashMap;
use std::time::Instant;
use num_integer::Integer;

use std::env;

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



        let disable_cache = env::var("CACHE_DISABLE").unwrap_or("false".to_string()).parse::<bool>().unwrap_or(false);
        let cap_str = env::var("CACHE_SIZE").unwrap_or("500000".to_string());
        let cap_per_str = env::var("CACHE_PERSISTENT_SIZE").unwrap_or("100000".to_string());
        let batch_str = env::var("BATCH_SIZE").unwrap_or("5000".to_string());

        let mut cap = cap_str.parse::<u32>().unwrap();
        let batch_size = batch_str.parse::<u32>().unwrap();
        let mut cap_per = cap_per_str.parse::<u32>().unwrap();

        if disable_cache{
            cap = 10000000;
            cap_per = cap;
        }

        info!("CACHE_DISABLE found/default: {} records", disable_cache);
        info!("CACHE_SIZE size found/default: {} records", cap);
        info!("CACHE_PERSISTENT_SIZE size found/default: {} records", cap_per);
        info!("BATCH_SIZE size found/default: {} documents", batch_size);

        let mut pre_index = PreIndex::with_capacity(cap,cap_per);
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
        let num_batches = num_docs.div_ceil(&batch_size);
        let mut processed_docs = 0;

        let mut timer;
        for batch in 0..num_batches {
            timer = Instant::now();
            let start_idx = batch * batch_size;
            let end_idx = ((batch + 1) * batch_size) - 1;

            let batch_documents_q = query("
                SELECT a.articleid, a.title, a.lastupdated, c.categories, c.links, c.text
                FROM article as a 
                INNER JOIN \"content\" as c
                    ON  a.articleid = c.articleid
                    AND a.articleid BETWEEN $1 AND $2
                ORDER BY a.articleid ASC
            ").bind(&start_idx)
            .bind(&end_idx)
            .fetch_all(&pool);
            
            let infoboxes_q = query("
            SELECT a.articleid, ARRAY_AGG( (CASE WHEN i.infoboxtype IS NULL THEN (NULL) ELSE (i.infoboxtype,i.body) END)) as infoboxes
            FROM article as a
            INNER JOIN infoboxes as i
                ON a.articleid = i.articleid
                AND a.articleid BETWEEN $1 AND $2
            GROUP BY a.articleid
            ORDER BY a.articleid ASC
            ").bind(&start_idx)
            .bind(&end_idx)
            .fetch_all(&pool);


            let citations_q = query("
            SELECT a.articleid, ARRAY_AGG(c.body) as citations
            FROM article as a
            INNER JOIN citations as c
                ON a.articleid = c.articleid
                AND a.articleid BETWEEN $1 AND $2
            GROUP BY a.articleid
            ORDER BY a.articleid ASC
            ").bind(&start_idx)
            .bind(&end_idx)
            .fetch_all(&pool);

            // let them run in parallel
            let batch_documents = batch_documents_q.await.unwrap();
            let mut infoboxes = infoboxes_q.await.unwrap()
                .into_iter().map(|i| (i.get("articleid"),i.get("infoboxes")))
                .collect::<HashMap<i64,Vec<(String,String)>>>();
            let mut citations = citations_q.await.unwrap()
                .into_iter().map(|c| (c.get("articleid"),c.get("citations")))
                .collect::<HashMap<i64,Vec<String>>>();
            
            batch_documents.into_iter().for_each(|d| {
                processed_docs += 1;
                let doc_id : i64 = d.get("articleid");
                let infoboxes : Option<Vec<(String,String)>> = infoboxes.remove(&doc_id);
                let citations : Option<Vec<String>> = citations.remove(&doc_id);
                let new_document = Box::new(Document {
                    doc_id: doc_id as u32,
                    categories: d.get("categories"),
                    main_text: d.get("text"),
                    article_links: d.get("links"),
                    title: d.get("title"),
                    last_updated_date: d.get("lastupdated"),
                    infoboxes: infoboxes.unwrap_or_default().into_iter().map(|v| Infobox {itype : v.0, text :v.1}).collect(),//v.get("infoboxes").map(|v| v).unwrap_or_default(),
                    citations: citations.unwrap_or_default().into_iter().map(|v| Citation {text :v}).collect(),//v.try_get("citations").unwrap_or_default(),
                });
                    
                if let Err(e) = pre_index.add_document(new_document){
                    let title : String = d.get("title");
                    error!("Error in adding document with {{id: {}, title: {}}}:{}",doc_id,title,e);
                }

            });



            info!("Building pre-index: {}% ({}s) - processed {} docs, cache size {}",(processed_docs as f32 / num_docs as f32) * 100.0,timer.elapsed().as_secs(),processed_docs,pre_index.cache_size());
        }

        pool.close().await;

        let idx = Index::from_pre_index(pre_index);

        Ok(Some(idx))
    }
}
