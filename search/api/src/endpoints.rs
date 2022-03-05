use crate::structs::SortType;
use crate::structs::{
    Document, RESTSearchData, Relation, RelationSearchOutput, RelationalSearchParameters,
    SearchParameters, UserFeedback,
};
use actix_web::ResponseError;
use actix_web::http::header::{HttpDate, ContentType};
use actix_web::{
    get,
    http::StatusCode,
    web::{Data, Json, Query},
    HttpResponse, Responder, Result,
};
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use index::errors::IndexError;
use index::index_structs::Posting;
use log::{debug, info};
use parser::errors::QueryError;
use parser::parser::parse_query;
use retrieval::search::{execute_query, preprocess_query, score_query, ScoredDocument};
use sqlx::postgres::PgPoolOptions;
use sqlx::Row;
use streaming_iterator::StreamingIterator;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display};
use std::fmt::Debug;
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
        f.debug_struct("APIError").field("code", &self.code).field("msg", &self.msg).field("hidden_msg", &self.hidden_msg).finish()
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

    fn new_user_error<T : Display, O : Display>( user_msg : &T, hidden_msg : &O) -> Self {
        APIError{
            code: StatusCode::UNPROCESSABLE_ENTITY,
            hidden_msg: hidden_msg.to_string(),
            msg: user_msg.to_string(),
        }
    }

    fn new_internal_error<T : Display + ?Sized>( hidden_msg : &T) -> Self {
        APIError{
            code: StatusCode::INTERNAL_SERVER_ERROR,
            hidden_msg: hidden_msg.to_string(),
            msg: "Something went wrong, please try again later!".to_string(),
        }
    }
}


impl std::error::Error for APIError {}

//TODO!:
//1) if index doesnt find id, return error to check implementation of index builder or retrieval.
//2) Check other parsing errors, throw them back to frontend
//3) adjust document scores based on tfidf parameter
// 4) Maybe caching user results could be good, but that is extra if we have time.
// 5) Optimise code (Less memory, instead of initialising another docs vector, use the one returned by score_query)
// 6) Make sure to return first page only if second page is not satisfied

// Endpoint for performing general wiki queries
#[get("/api/v1/search")]
pub async fn search(
    data: Data<RESTSearchData>,
    q: Query<SearchParameters>,
) -> Result<impl Responder, APIError> {

    let timer = Instant::now();

    //Initialise Database connection to retrieve article title and abstract for each document found for the query
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&data.connection_string)
        .await
        .map_err(|_e| APIError::new_internal_error("Failed to initialise connection with postgres"))?;

    // construct + execute query
    let idx = data
        .index_rest
        .read()
        .map_err(|e| APIError::new_internal_error(&e))?;
    let (_, ref mut query) = parse_query(&q.query)
        .map_err(|e| APIError::new_user_error(&e,&e))?;
    
    preprocess_query(query).map_err(|e| APIError::new_user_error(&e,&e))?;

    let mut postings = execute_query(query, &idx).cloned().collect::<Vec<Posting>>();

    // score documents if necessary and sort appropriately
    let ordered_docs: Vec<ScoredDocument> = match q.sort_by {
        SortType::Relevance => {
            let mut scored_documents = score_query(query, &idx, &mut postings);
            scored_documents.sort_unstable_by(|doc1, doc2| {
                doc2.score
                    .partial_cmp(&doc1.score)
                    .unwrap_or(Ordering::Equal)
            });
            scored_documents
                .into_iter() // consumes scored_documents
                .skip((q.results_per_page.0 * (q.page.0 - 1)) as usize)
                .take(q.results_per_page.0 as usize)
                .collect()
        }
        SortType::LastEdited => {
            postings.dedup_by_key(|v| v.document_id);
            postings.sort_by_cached_key(|Posting { document_id, .. }| {
                idx.get_last_updated_date(*document_id)
            });
            postings
                .into_iter() // consumes postings
                .skip((q.results_per_page.0 * (q.page.0 - 1)) as usize)
                .take(q.results_per_page.0 as usize)
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
            let pool_cpy = pool.clone();
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
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<Result<Document, APIError>>>()
        .await
        .into_iter()
        .collect::<Result<Vec<Document>, APIError>>()?; // fail on a single internal error

    info!("Query: {} took: {}us",&q.query, timer.elapsed().as_micros());

    Ok(Json(future_documents))
}

/// Endpoint for performing relational searches stretching from a given root
#[get("/api/v1/relational")]
pub async fn relational(
    data: Data<RESTSearchData>,
    q: Query<RelationalSearchParameters>,
) -> Result<impl Responder, APIError> {
    let timer = Instant::now();

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&data.connection_string)
        .await
        .map_err(|e| APIError::new_internal_error(&e))?;

    // construct + execute query
    let root_article = sqlx::query(
        "SELECT a.articleid
        From article as a
        where a.title=$1",
    )
    .bind(q.root.clone())
    .fetch_one(&pool)
    .await
    .map_err(|e| APIError::new_user_error(
        &format!("The root article provided `{}` is not a valid root article title",q.root),&e))?;
    

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
        q.hops,
        q.query
            .clone()
            .map(|v| format!(",{}", v))
            .unwrap_or("".to_string()));
    debug!("{:?}", query_string);
    let (_, ref mut query) = parse_query(&query_string)
        .map_err(|e| APIError::new_user_error(&e,&e))?;


    preprocess_query(query).map_err(|e| APIError::new_user_error(&e,&e))?;

    let mut postings = execute_query(query, &idx).cloned().collect::<Vec<Posting>>();
    let scored_documents = score_query(query, &idx, &mut postings); // page rank and stuff

    // keep track of the translations between titles and ids
    // as well as the documents present in the query for later
    // get documents
    let documents = scored_documents
        .iter()
        .map(|doc| {
            let pool_cpy = pool.clone();
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
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<Result<Document, APIError>>>()
        .await
        .into_iter()
        .collect::<Result<Vec<Document>, APIError>>()?; // fail on a single internal error

    let mut title_map: HashMap<u32, &str> = HashMap::with_capacity(documents.len());
    documents.iter().for_each(|d| {
        title_map.insert(d.id, &d.title);
    });

    // find links
    // TODO: this is extremely inefficient, we only need links between documents retrieved
    // also there may be duplicates, need to retrieve this while crawling the graph
    let relations: HashSet<Relation> = scored_documents
        .iter()
        .flat_map(|ScoredDocument { doc_id, score: _ }| {
            debug!("DOC: {}",doc_id);
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

    info!("Relational Query: {:?} took: {}us",&q.query, timer.elapsed().as_micros());

    Ok(Json(RelationSearchOutput {
        documents: documents,
        relations: relations.into_iter().collect(),
    }))
}

#[get("/api/v1/feedback")]
pub async fn feedback(_q: Query<UserFeedback>) -> Result<impl Responder> {
    Ok(HttpResponse::Ok().finish())
}
