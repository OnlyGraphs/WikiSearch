use crate::api::structs::{
    Document, Relation, RelationSearchOutput, RelationalSearchParameters, SearchParameters,
    UserFeedback,
};
use actix_web::{
    get,
    web::{Json, Query},
    HttpResponse, Responder, Result,
};

/// Endpoint for performing general wiki queries
#[get("/api/v1/search")]
pub async fn search(_q: Query<SearchParameters>) -> Result<impl Responder> {
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

    let docs = vec![document1, document2];

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


