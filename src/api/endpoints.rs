use crate::api::structs::default_results_per_page;
use crate::api::structs::{
    Document, RESTSearchData, Relation, RelationSearchOutput, RelationalSearchParameters,
    SearchParameters, UserFeedback,
};
use crate::index::index::{BasicIndex, Index};
use crate::index_structs::Posting;
use crate::parser::parser::parse_query;
use crate::search::search::execute_query;
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
pub struct MyError(String);
impl std::fmt::Display for MyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Oh no. something happened") //TODO!: Write a meaningful error
    }
}
impl ResponseError for MyError {}

fn get_retrieved_documents(postings: Vec<Posting>) -> Vec<u32> {
    let mut doc_retrieved_set = HashSet::new();
    let mut doc_vector = Vec::new();
    for post in postings {
        if doc_retrieved_set.get(&post.document_id) == None {
            doc_retrieved_set.insert(post.document_id);
            doc_vector.push(post.document_id);
        }
    }
    return doc_vector;
}

//TODO!:
//1) if index doesnt find id, return error to check implementation of index builder or retrieval.
//2) Check other parsing errors, throw them back to frontend
//3) adjust document scores based on tfidf parameter
// 4) Maybe caching user results could be good, but that is extra if we have time.

// Endpoint for performing general wiki queries
#[get("/api/v1/search")]
pub async fn search(
    data: Data<RESTSearchData>,
    _q: Query<SearchParameters>,
) -> Result<impl Responder> {
    debug!("Query Before Parsing: {:?}", &_q.query);
    let (_, query) = parse_query(&_q.query).unwrap();

    debug!("Query Form After Parsing: {:?}", query);

    let results_per_page = _q.results_per_page.unwrap();
    debug!("Results Per Page: {:?}", results_per_page);

    let page = _q.page.unwrap();
    debug!("Current Page Number: {:?}", page);

    let idx = data.index_rest.read().unwrap();
    let postings = execute_query(query, &idx);
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&data.connection_string)
        .await
        .expect("DB error"); //TODO! Handle error appropriately

    let mut docs = Vec::new();
    let retrieved_doc_ids = get_retrieved_documents(postings);

    let mut doc_index: usize = ((page - 1) * (results_per_page as u32)).try_into().unwrap();
    let results_per_page: usize = (page * (results_per_page as u32)).try_into().unwrap();

    while doc_index < retrieved_doc_ids.len() && doc_index + 1 <= results_per_page {
        let articleid = retrieved_doc_ids[doc_index];
        debug!("Document: {:?}", articleid);

        let sql = sqlx::query(
            "SELECT a.title, c.abstracts
        From article as a, \"content\" as c
        where a.articleid= $1 AND a.articleid = c.articleid",
        )
        .bind(articleid)
        .fetch_one(&pool)
        .await
        .expect("Query error"); //TODO!: Handle error more appropriately
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
