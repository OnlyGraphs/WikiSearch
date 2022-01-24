use actix_web::{get, web, Responder, Result};
use super::structs;

/// Endpoint for performing general wiki queries
#[get("/api/v1/search")]
pub async fn search(_q: web::Query<structs::SearchParameters>) -> Result<impl Responder>{ 

    let mut docs:Vec<structs::Document> = Vec::new();

    let document1 = structs::Document{
        title: "April".to_string(),
        article_abstract: "April is the fourth month of the year in the Gregorian calendar, the fifth in the early Julian, the first of four months to have a length of 30 days, and the second of five months to have a length of less than 31 days.".to_string(),
        score: 0.5,
    };

    let document2 = structs::Document{
        title: "May".to_string(),
        article_abstract: "May is the fifth month of the year in the Julian and Gregorian calendars and the third of seven months to have a length of 31 days.".to_string(),
        score: 0.6,
    };

    docs.push(document1);
    docs.push(document2);

    Ok(web::Json(docs))
}

/// Endpoint for performing relational searches stretching from a given root
#[get("/api/v1/relational")]
pub async fn relational(_q: web::Query<structs::RelationalSearchParameters>) -> Result<impl Responder>{ 


    let mut docs:Vec<structs::Document> = Vec::new();

    let document1 = structs::Document{
        title: "April".to_string(),
        article_abstract: "April is the fourth month of the year in the Gregorian calendar, the fifth in the early Julian, the first of four months to have a length of 30 days, and the second of five months to have a length of less than 31 days.".to_string(),
        score: 0.5,
    };

    let document2 = structs::Document{
        title: "May".to_string(),
        article_abstract: "May is the fifth month of the year in the Julian and Gregorian calendars and the third of seven months to have a length of 31 days.".to_string(),
        score: 0.6,
    };

    docs.push(document1);
    docs.push(document2);

    let mut relations:Vec<structs::Relation> = Vec::new();

    let relation1 = structs::Relation{
        source: "April".to_string(),
        destination: "May".to_string(),
    };

    let relation2 = structs::Relation{
        source: "May".to_string(),
        destination: "April".to_string(),
    };
    
    relations.push(relation1);
    relations.push(relation2);

    let out = structs::RelationSearchOutput{
        documents: docs,
        relations: relations,
    };
    
    Ok(web::Json(out))
}
    