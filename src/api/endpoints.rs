use crate::api::structs::{
    Document, RESTSearchData, Relation, RelationSearchOutput, RelationalSearchParameters,
    SearchParameters, UserFeedback,
};
use crate::index::index::{BasicIndex, Index};
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

//TODO!: 1) if index doesnt find id, return error to check implementation of index builder or retrieval.
//2) Check other parsing errors, throw them back to frontend

// Endpoint for performing general wiki queries
#[get("/api/v1/search")]
pub async fn search(
    data: Data<RESTSearchData>,
    _q: Query<SearchParameters>,
) -> Result<impl Responder> {
    let (nxt, query) = parse_query(&_q.query).unwrap();
    debug!("{:?}", nxt);
    let idx = data.index_rest.read().unwrap();
    debug!("Query: {:?}", query);
    let postings = execute_query(query, &idx);
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&data.connection_string)
        .await
        .expect("DB error"); //TODO! Handle error appropriately

    let mut docs = Vec::new();

    let mut doc_retrieved_set = HashSet::new();
    for post in postings.iter() {
        match doc_retrieved_set.get(&post.document_id) {
            Some(x) => continue,
            None => doc_retrieved_set.insert(post.document_id),
        };
        info!("Document: {:?}", post.document_id);

        let sql = sqlx::query(
            "SELECT a.title, c.abstracts
        From article as a, \"content\" as c
        where a.articleid= $1 AND a.articleid = c.articleid",
        )
        .bind(post.document_id)
        .fetch_one(&pool)
        .await
        .expect("Query error");
        // .map_err(|e| {
        //     println!("error is {}", e);
        //     MyError(String::from("oh no. anyways"))
        // })?; //TODO!: Handle error more appropriately
        // .expect("Query error"); //TODO!: Handle error more appropriately
        let title: String = sql.try_get("title").unwrap_or_default();
        let article_abstract: String = sql.try_get("abstracts").unwrap_or_default();

        docs.push(Document {
            title: title,
            article_abstract: article_abstract,
            score: 0.0,
        });
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
