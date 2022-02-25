use crate::index::errors::IndexError;
use std::fmt::Debug;
use futures::StreamExt;
use crate::search::search::ScoredDocument;
use crate::index_structs::Posting;
use futures::stream::FuturesUnordered;
use crate::parser::errors::QueryError;
use actix_web::ResponseError;
use crate::search::search::preprocess_query;
use crate::api::structs::{
    Document, RESTSearchData, Relation, RelationSearchOutput, RelationalSearchParameters,
    SearchParameters, UserFeedback,
};
use crate::parser::parser::parse_query;
use crate::search::search::{execute_query, score_query};
use crate::structs::SortType;
use actix_web::{
    get,
    web::{Data, Json, Query},
    HttpResponse, Responder, Result,http::StatusCode
};
use sqlx::Row;
use sqlx::{postgres::PgPoolOptions};
use log::{debug};
use std::fmt;
use futures::future;
use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct APIError {
    pub code: StatusCode,
    pub msg: String,
}

impl fmt::Display for APIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{:?}",self.msg)
    }
}

impl ResponseError for APIError{

    fn status_code(&self) -> StatusCode{
        self.code
    }
}

impl APIError{
    fn from_status_code(s : StatusCode) -> Self {
        APIError{
            code: s,
            msg: "".to_string()
        }
    }

    fn from_printable<T : Debug>(p : T, s : StatusCode) -> Self{
        APIError{
            code: s,
            msg: format!("{:?}",p)
        }
    }
}

impl From<QueryError> for APIError {
    fn from(e: QueryError) -> Self {
        APIError{
            code: StatusCode::UNPROCESSABLE_ENTITY,
            msg: format!("{:?}",e)
        }
    }
}

impl From<IndexError> for APIError {
    fn from(e: IndexError) -> Self {
        APIError{
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: format!("{:?}",e)
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
    //Initialise Database connection to retrieve article title and abstract for each document found for the query
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&data.connection_string)
        .await
        .map_err(|e| APIError::from_status_code(StatusCode::INTERNAL_SERVER_ERROR))?; 


    // construct + execute query
    let idx = data.index_rest.read()
        .map_err(|e| APIError::from_printable(e, StatusCode::UNPROCESSABLE_ENTITY))?;
    let (_,ref mut query) = parse_query(&q.query)
        .map_err(|e| APIError::from_printable(e, StatusCode::UNPROCESSABLE_ENTITY))?;
    preprocess_query(query)?;

    let mut postings = execute_query(query, &idx);

    // score documents if necessary and sort appropriately
    let ordered_docs : Vec<ScoredDocument> = match q.sortby {
        SortType::Relevance => {
            let mut scored_documents = score_query(query, &idx, &postings);
            scored_documents.sort_unstable_by(|doc1, doc2| doc2.score.partial_cmp(&doc1.score).unwrap_or(Ordering::Equal));
            scored_documents.into_iter() // consumes scored_documents
                .skip((q.results_per_page.0 * (q.page.0 - 1)) as usize)
                .take(q.results_per_page.0 as usize)
                .collect()
            
        },
        SortType::LastEdited => {
            postings.sort_by_cached_key(|Posting{document_id , ..}| idx.get_last_updated_date(*document_id));
            postings.into_iter() // consumes postings
                .skip((q.results_per_page.0 * (q.page.0 - 1)) as usize)
                .take(q.results_per_page.0 as usize)
                .map(|p| ScoredDocument{doc_id: p.document_id, score: 1.0})
                .collect()
        }

    };

    let future_documents = ordered_docs.into_iter() // consumes ordered_docs
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
                    .map_err(|_| APIError::from_status_code(StatusCode::INTERNAL_SERVER_ERROR))?;

                    let title : String = sql.try_get("title").map_err(|_| APIError::from_status_code(StatusCode::INTERNAL_SERVER_ERROR))?;
                    let abstracts : String = sql.try_get("abstracts").map_err(|_| APIError::from_status_code(StatusCode::INTERNAL_SERVER_ERROR))?;
                    Ok::<Document,APIError>(Document {
                        title: title,
                        article_abstract: abstracts,
                        score: doc.score
                    })
                }
            }
            )
        .collect::<FuturesUnordered<_>>()
        .collect::<Vec<Result<Document,APIError>>>()
        .await
        .into_iter()
        .collect::<Result<Vec<Document>,APIError>>()?; // fail on a single internal error

    Ok(Json(future_documents))
}

/// Endpoint for performing relational searches stretching from a given root
#[get("/api/v1/relational")]
pub async fn relational(data: Data<RESTSearchData>,q: Query<RelationalSearchParameters>) -> Result<impl Responder, APIError> {

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&data.connection_string)
        .await
        .map_err(|e| APIError::from_status_code(StatusCode::INTERNAL_SERVER_ERROR))?; 

    // construct + execute query
    let idx = data.index_rest.read()
        .map_err(|e| APIError::from_printable(e, StatusCode::UNPROCESSABLE_ENTITY))?;
    let (_,ref mut query) = parse_query(
            &format!("#LINKEDTO, {},{} {}",q.root,q.hops,q.query.clone().map(|v| 
                format!(",{}",v)).unwrap_or("".to_string())))
        .map_err(|e| APIError::from_printable(e, StatusCode::UNPROCESSABLE_ENTITY))?;

    debug!("Query: {:?}",query);

    preprocess_query(query)?;

    let mut postings = execute_query(query, &idx);
    let mut scored_documents = score_query(query, &idx, &postings); // page rank and stuff
    
    // get documents 
    let documents = scored_documents.iter() 
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
                .map_err(|_| APIError::from_status_code(StatusCode::INTERNAL_SERVER_ERROR))?;

                let title : String = sql.try_get("title").map_err(|_| APIError::from_status_code(StatusCode::INTERNAL_SERVER_ERROR))?;
                let abstracts : String = sql.try_get("abstracts").map_err(|_| APIError::from_status_code(StatusCode::INTERNAL_SERVER_ERROR))?;
                Ok::<Document,APIError>(Document {
                    title: title,
                    article_abstract: abstracts,
                    score: doc.score
                })
            }
        }
        )
    .collect::<FuturesUnordered<_>>()
    .collect::<Vec<Result<Document,APIError>>>()
    .await
    .into_iter()
    .collect::<Result<Vec<Document>,APIError>>()?; // fail on a single internal error

    // find links 
    // TODO: this is extremely inefficient, we only need links between documents retrieved
    // also there may be duplicates, need to retrieve this while crawling the graph
    let relations : Vec<Relation> = scored_documents.iter().flat_map(|ScoredDocument{doc_id, score}|{
        debug!("{:?},{:?}", *doc_id,idx.get_links(*doc_id));
        idx.get_links(*doc_id).unwrap()
            .iter().map(|id| { debug!("{:?}",id); Relation {
                source: idx.id_to_title(*doc_id).unwrap().to_string(),
                destination: idx.id_to_title(*id).unwrap().to_string(),
            }})
            .chain(idx.get_incoming_links(*doc_id).iter().map(|id| Relation{
                source: idx.id_to_title(*id).unwrap().to_string(),
                destination: idx.id_to_title(*doc_id).unwrap().to_string()
            }))
            .collect::<Vec<Relation>>()
        }
    ).collect();

    Ok(Json(RelationSearchOutput{
        documents: documents,
        relations: relations,
    }))
}

#[get("/api/v1/feedback")]
pub async fn feedback(_q: Query<UserFeedback>) -> Result<impl Responder> {
    Ok(HttpResponse::Ok().finish())
}
