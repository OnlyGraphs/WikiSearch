use crate::structs::SortType;
use crate::structs::{
    Document, RESTSearchData, Relation, RelationSearchOutput, RelationalSearchParameters,
    SearchParameters, UserFeedback,
};
use crate::{RelationDocument, SearchOutput};
use actix_web::http::header::ContentType;
use actix_web::ResponseError;
use actix_web::{
    get,
    http::StatusCode,
    web::{Data, Json, Query},
    HttpResponse, Responder, Result,
};
use futures::stream::{FuturesUnordered, FuturesOrdered};
use futures::StreamExt;

use index::index_structs::Posting;
use log::{debug, info};

use parser::parser::parse_query;
use retrieval::search::{execute_query, preprocess_query, score_query, ScoredDocument};
use retrieval::{execute_relational_query, ScoredRelationDocument};
use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
use std::cmp::{min, Ordering, max};
use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt::{self, Display};

use retrieval::correct_query;
use std::time::Instant;
pub struct APIError {
    pub code: StatusCode,
    pub msg: String,
    pub hidden_msg: String,
}

impl fmt::Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.msg)
    }
}

impl fmt::Debug for APIError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("APIError")
            .field("code", &self.code)
            .field("msg", &self.msg)
            .field("hidden_msg", &self.hidden_msg)
            .finish()
    }
}

impl ResponseError for APIError {
    fn status_code(&self) -> StatusCode {
        self.code
    }
}

impl APIError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }


    pub fn new_user_error<T: Display, O: Display>(user_msg: &T, hidden_msg: &O) -> Self {
        APIError {
            code: StatusCode::UNPROCESSABLE_ENTITY,
            hidden_msg: hidden_msg.to_string(),
            msg: user_msg.to_string(),
        }
    }

    pub fn new_internal_error<T: Display + ?Sized>(hidden_msg: &T) -> Self {
        APIError {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            hidden_msg: hidden_msg.to_string(),
            msg: "Something went wrong, please try again later!".to_string(),
        }
    }
}


impl std::error::Error for APIError {}

// Endpoint for performing general wiki queries
#[get("/api/v1/search")]
pub async fn search(
    data: Data<RESTSearchData>,
    q: Query<SearchParameters>,
) -> Result<impl Responder, APIError> {
    let timer_whole = Instant::now();

    info!("received query: {}", q.query);
    if q.query.len() > 255 {
        let msg = "Query is too long, please shorten it before trying again.".to_string(); 
        return Err(APIError::new_user_error(
            &msg,&msg
        ))
    }

    // construct + execute query
    let idx = data
        .index_rest
        .read()
        .map_err(|e| APIError::new_internal_error(&e))?;
    let (_, ref mut query) = parse_query(&q.query)
        .map_err(|e| APIError::new_user_error(&format!("Your query: {} is not valid, please form a valid query.",q.query),&e))?;
    


    let mut timer = Instant::now();
    preprocess_query(query)
        .map_err(|e| APIError::new_user_error(
            &format!("Your query: {} is not valid, please form a valid query.",q.query),
            &e))?;
    info!(
        "preprocessed query: {:?}, {}s",
        query,
        timer.elapsed().as_secs_f32()
    );

    timer = Instant::now();
    let postings_query = execute_query(query, &idx);
    info!("executed query: {}s", timer.elapsed().as_secs_f32());

    timer = Instant::now();
    let suggested_query = correct_query(query, &idx);
    info!("{}", format!("Suggested Query:  {}", suggested_query));
    info!("Corrected query: {}s", timer.elapsed().as_secs_f32());

    timer = Instant::now();
    let mut postings = postings_query.collect::<Vec<Posting>>();

    let capped_max_results = min(q.results_per_page.0, 150);
    // score documents if necessary and sort appropriately
    let ordered_docs: Vec<ScoredDocument> = match q.sort_by {
        SortType::Relevance => {
            let mut scored_documents = score_query(query, &idx, &mut postings);
            scored_documents.sort_unstable_by(|doc1, doc2| {
                doc2.score
                    .partial_cmp(&doc1.score)
                    .unwrap_or(Ordering::Less)
            });
            scored_documents
                .into_iter() // consumes scored_documents
                .skip((capped_max_results * (q.page.0 - 1)) as usize)
                .take(capped_max_results as usize)
                .collect()
        }
        SortType::LastEdited => {
            postings.dedup_by_key(|v| v.document_id);
            postings.sort_by_cached_key(|Posting { document_id, .. }| {
                idx.get_last_updated_date(*document_id)
            });
            postings
                .into_iter() // consumes postings
                .skip((capped_max_results * (q.page.0 - 1)) as usize)
                .take(capped_max_results as usize)
                .map(|p| ScoredDocument {
                    doc_id: p.document_id,
                    score: 1.0,
                })
                .collect()
        }
    };

    let future_documents = ordered_docs
        .into_iter() // consumes ordered_docs
        .map(|doc| {
            let pool_cpy = data.pool.clone();
            async move {
                let sql = sqlx::query(
                    "SELECT a.title, c.abstracts
                    From article as a, \"content\" as c
                    where a.articleid= $1 AND a.articleid = c.articleid",
                )
                .bind(doc.doc_id as i64)
                .fetch_one(&pool_cpy)
                .await
                .map_err(|e| APIError::new_internal_error(&e))?;

                let title: String = sql
                    .try_get("title")
                    .map_err(|e| APIError::new_internal_error(&e))?;
                let abstracts: String = sql
                    .try_get("abstracts")
                    .map_err(|e| APIError::new_internal_error(&e))?;
                Ok::<Document, APIError>(Document {
                    id: doc.doc_id,
                    title: title,
                    article_abstract: abstracts,
                    score: doc.score,
                })
            }
        })
        .collect::<FuturesOrdered<_>>()
        .collect::<Vec<Result<Document, APIError>>>()
        .await
        .into_iter()
        .collect::<Result<Vec<Document>, APIError>>()?; // fail on a single internal error

    info!("sorted query: {}s", timer.elapsed().as_secs_f32());

    info!(
        "Query: {} took: {}s",
        &q.query,
        timer_whole.elapsed().as_secs_f32()
    );

    Ok(Json(SearchOutput {
        documents: future_documents,
        domain: env::var("DOMAIN").unwrap_or("en".to_string()),
        suggested_query: suggested_query,
    }))
}

/// Endpoint for performing relational searches stretching from a given root
#[get("/api/v1/relational")]
pub async fn relational(
    data: Data<RESTSearchData>,
    q: Query<RelationalSearchParameters>,
) -> Result<impl Responder, APIError> {
    let timer = Instant::now();

    if q.query.as_ref().unwrap_or(&"".to_string()).len() > 255 {
        let msg = "Query is too long, please shorten it before trying again.".to_string(); 
        return Err(APIError::new_user_error(
            &msg,&msg
        ))
    }

    // construct + execute query
    let root_article = sqlx::query(
        "SELECT a.articleid
        From article as a
        where a.title=$1",
    )
    .bind(q.root.clone())
    .fetch_one(&data.pool)
    .await
    .map_err(|e| {
        APIError::new_user_error(
            &format!(
                "The root article provided `{}` is not a valid root article title",
                q.root
            ),
            &e,
        )
    })?;

    let root_id: i64 = root_article
        .try_get("articleid")
        .map_err(|e| APIError::new_internal_error(&e))?;

    let idx = data
        .index_rest
        .read()
        .map_err(|e| APIError::new_internal_error(&e))?;

    let query_string = format!(
        "#LINKSTO, {},{} {}",
        root_id,
        max(q.hops,5), // max out hops
        q.query
            .clone()
            .map(|v| format!(",{}", v))
            .unwrap_or("".to_string())
    );

    let (_, ref mut query) =
        parse_query(&query_string).map_err(|e| APIError::new_user_error(&e, &e))?;

    preprocess_query(query).map_err(|e| APIError::new_user_error(&e, &e))?;

    let capped_max_results = min(q.max_results.0, 150) as usize;

    let mut scored_documents = execute_relational_query(query, &idx);
    scored_documents.sort_by(|a, b| {
        a.hops
            .partial_cmp(&b.hops)
            .unwrap_or(std::cmp::Ordering::Equal)
    }); // TODO; hmm
    scored_documents = scored_documents
        .into_iter()
        .take(capped_max_results)
        .collect::<Vec<ScoredRelationDocument>>();

    // keep track of the translations between titles and ids
    // as well as the documents present in the query for later
    // get documents
    let documents = scored_documents
        .iter()
        .map(|doc| {
            let pool_cpy = data.pool.clone();
            async move {
                let sql = sqlx::query(
                    "SELECT a.title, c.abstracts
                From article as a, \"content\" as c
                where a.articleid= $1 AND a.articleid = c.articleid",
                )
                .bind(doc.doc_id as i64)
                .fetch_one(&pool_cpy)
                .await
                .map_err(|e| APIError::new_internal_error(&e))?;

                let title: String = sql
                    .try_get("title")
                    .map_err(|e| APIError::new_internal_error(&e))?;
                let abstracts: String = sql
                    .try_get("abstracts")
                    .map_err(|e| APIError::new_internal_error(&e))?;

                Ok::<RelationDocument, APIError>(RelationDocument {
                    id: doc.doc_id,
                    title: title,
                    article_abstract: abstracts,
                    score: doc.score,
                    hops: doc.hops,
                })
            }
        })
        .collect::<FuturesOrdered<_>>()
        .collect::<Vec<Result<RelationDocument, APIError>>>()
        .await
        .into_iter()
        .collect::<Result<Vec<RelationDocument>, APIError>>()?; // fail on a single internal error

    let mut title_map: HashMap<u32, &str> = HashMap::with_capacity(documents.len());
    documents.iter().for_each(|d| {
        title_map.insert(d.id, &d.title);
    });

    // find links
    // TODO: this is extremely inefficient, we only need links between documents retrieved
    // also there may be duplicates, need to retrieve this while crawling the graph
    let relations: HashSet<Relation> = scored_documents
        .iter()
        .flat_map(|ScoredRelationDocument { doc_id, .. }| {
            idx.get_links(*doc_id)
                .iter()
                .filter_map(|target| {
                    if !title_map.contains_key(target) {
                        None
                    } else {
                        Some(Relation {
                            source: title_map.get(&doc_id).unwrap().to_string(),
                            destination: title_map.get(&target).unwrap().to_string(),
                        })
                    }
                })
                .chain(idx.get_incoming_links(*doc_id).iter().filter_map(|source| {
                    if !title_map.contains_key(source) {
                        None
                    } else {
                        Some(Relation {
                            source: title_map.get(&source).unwrap().to_string(),
                            destination: title_map.get(&doc_id).unwrap().to_string(),
                        })
                    }
                }))
                .collect::<Vec<Relation>>()
        })
        .collect();

    info!(
        "Relational Query: {:?} took: {}s",
        &q.query,
        timer.elapsed().as_secs_f32()
    );

    Ok(Json(RelationSearchOutput {
        documents: documents,
        relations: relations.into_iter().collect(),
        domain: env::var("DOMAIN").unwrap_or("en".to_string()),
        suggested_query: "".to_string(),
    }))
}

#[get("/api/v1/feedback")]
pub async fn feedback(_q: Query<UserFeedback>) -> Result<impl Responder> {
    Ok(HttpResponse::Ok().finish())
}
