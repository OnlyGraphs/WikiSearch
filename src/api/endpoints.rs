use crate::api::structs::default_results_per_page;
use crate::api::structs::{
    Document, RESTSearchData, Relation, RelationSearchOutput, RelationalSearchParameters,
    SearchParameters, UserFeedback,
};
use crate::index::index::{BasicIndex, Index};
use crate::index_structs::Posting;
use crate::parser::parser::parse_query;
use crate::search::search::{execute_query, score_query};
use crate::structs::SortType;
use actix_web::{
    get,
    web::{Data, Json, Query},
    HttpResponse, Responder, ResponseError, Result,
};
use sqlx::Row;
use sqlx::{postgres::PgPoolOptions, query, query_scalar};
use std::collections::HashSet;
use std::{
    env,
    sync::{Arc, RwLock},
};

use log::{debug, info};

#[derive(Debug)]
pub enum APIError {
    DatabaseError,
    QueryFormattingError,
    EmptyQueryError,
}

impl std::fmt::Display for APIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Oh no. something happened") //TODO!: Write a meaningful error
    }
}
impl ResponseError for APIError {}

// impl sqlx::Error for APIError {}

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
    _q: Query<SearchParameters>,
) -> Result<impl Responder, APIError> {
    debug!("Query Before Parsing: {:?}", &_q.query);
    let (_, query) = parse_query(&_q.query).unwrap();
    debug!("Query Form After Parsing: {:?}", query);

    let results_per_page = _q.results_per_page.unwrap();
    debug!("Results Per Page: {:?}", results_per_page);

    let page = _q.page.unwrap();
    debug!("Current Page Number: {:?}", page);

    let sortby = _q.sortby.as_ref().unwrap();
    debug!("Sort by: {:?}", sortby);

    let idx = data.index_rest.read().unwrap();
    let postings = execute_query(query.clone(), &idx);
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&data.connection_string)
        .await
        .expect("DB error"); //TODO! Handle error appropriately

    let mut scored_documents = score_query(query, &idx, &postings);
    debug!("Number of documents found: {:?}", scored_documents.len());
    //Sort depending on type
    match sortby {
        SortType::Relevance => scored_documents.sort_by_key(|doc| doc.get_score()),
        SortType::LastEdited => scored_documents.sort_by_key(|doc| doc.get_date()),
    };
    assert_eq!(scored_documents, scored_documents);

    //Compute which documents to return
    //TODO! Make sure to return first page
    let mut doc_index: usize = ((page - 1) * (results_per_page as u32)).try_into().unwrap();
    let doc_index_end: usize = (page * (results_per_page as u32)).try_into().unwrap();
    debug!("Start Doc: {:?}", doc_index);
    debug!("End Doc: {:?}", doc_index_end);

    let mut docs = Vec::new();
    while doc_index < scored_documents.len() && doc_index + 1 <= doc_index_end {
        let articleid = scored_documents[doc_index].get_doc_id();
        debug!("Document: {:?}", articleid);
        debug!("Date: {:?}", scored_documents[doc_index].get_date());

        let sql = sqlx::query(
            "SELECT a.title, c.abstracts
        From article as a, \"content\" as c
        where a.articleid= $1 AND a.articleid = c.articleid",
        )
        .bind(articleid)
        .fetch_one(&pool)
        .await
        .expect("Query error"); //TODO! Handle error appropriately

        let title: String = sql.try_get("title").unwrap_or_default();

        let article_abstract: String = sql.try_get("abstracts").unwrap_or_default();

        docs.push(Document {
            title: title,
            article_abstract: article_abstract,
            score: 0.0, //TODO! Adjust score based on tfidf
        });
        //Go to the next posting
        doc_index += 1;
    }
    debug!("Number of results returned: {:?}", docs.len());

    Ok(Json(docs))
}

/// Endpoint for performing relational searches stretching from a given root
#[get("/api/v1/relational")]
pub async fn relational(_q: Query<RelationalSearchParameters>) -> Result<impl Responder> {
    let document1 = Document{
        title: "April".to_string(),
        article_abstract: "April is the fourth month of the year in the Gregorian calendar, the fifth in the early Julian, the first of four months to have a length of 30 days, and the second of five months to have a length of less than 31 days.".to_string(),
        score: 0.5,
    };

    let document2 = Document{
        title: "May".to_string(),
        article_abstract: "May is the fifth month of the year in the Julian and Gregorian calendars and the third of seven months to have a length of 31 days.".to_string(),
        score: 0.6,
    };

    let relation1 = Relation {
        source: "April".to_string(),
        destination: "May".to_string(),
    };

    let relation2 = Relation {
        source: "May".to_string(),
        destination: "April".to_string(),
    };

    let docs = vec![document1, document2];
    let relations = vec![relation1, relation2];

    let out = RelationSearchOutput {
        documents: docs,
        relations: relations,
    };

    Ok(Json(out))
}

#[get("/api/v1/feedback")]
pub async fn feedback(_q: Query<UserFeedback>) -> Result<impl Responder> {
    Ok(HttpResponse::Ok().finish())
}
