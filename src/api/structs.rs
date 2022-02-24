use serde::Deserialize;
use serde::Serialize;

use crate::index::index::Index;
use std::sync::{Arc, RwLock};

/// Represents the type of order to be imposed on list of documents
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum SortType {
    Relevance,
    LastEdited, //Sort in descending order of dates
}

impl Default for SortType {
    fn default() -> Self {
        SortType::Relevance
    }
}

/// Represents the parameters of a given standard search
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchParameters {
    pub query: String,

    #[serde(default)]
    pub sortby: SortType,

    #[serde(default)]
    pub page: DefaultPage,

    #[serde(default)]
    pub results_per_page: ResultsCount,
}

/// Represents the parameters of a given relational search
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationalSearchParameters {
    pub hops: u8,
    pub root: String,

    #[serde(default)]
    pub query: Option<String>,

    #[serde(default)]
    pub max_results: ResultsCount,
}

/// Represents a piece of feedback related to a user
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserFeedback {
    pub query: String,
    pub result_page: u8,

    #[serde(default)]
    pub chosen_result: Option<String>,
}

/// Represents a wikipedia article
#[derive(Serialize, Debug)]

pub struct Document {
    pub title: String,
    pub score: f64,

    #[serde(rename = "abstract")]
    pub article_abstract: String,
}

/// Represents a relation between two articles
/// where source is the origin of a link
/// and destination is the destination of the link
#[derive(Serialize)]
pub struct Relation {
    pub source: String,
    pub destination: String,
}

/// Represents a collection of documents and relations
#[derive(Serialize)]
pub struct RelationSearchOutput {
    pub documents: Vec<Document>,
    pub relations: Vec<Relation>,
}




#[derive(Deserialize, Debug, Clone)]
pub struct ResultsCount(pub u16);
impl Default for ResultsCount {
    fn default() -> Self {
        ResultsCount(20)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct DefaultPage(pub u16);
impl Default for DefaultPage {
    fn default() -> Self {
        DefaultPage(1)
    }
}


#[derive(Debug)]
pub struct RESTSearchData {
    pub index_rest: Arc<RwLock<Box<dyn Index>>>,
    pub connection_string: String, //Used to query Database for metadata results like Title or Abstracts
}
