use serde::Deserialize;
use serde::Serialize;


/// Represents the type of order to be imposed on list of documents
#[derive(Debug,Deserialize)]
#[serde(rename_all = "camelCase")] 
pub enum SortType {
    Relevance,
    LastEdited,
}

/// Represents the parameters of a given standard search
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")] 
pub struct SearchParameters {

    pub query: String,

    #[serde(default= "default_sortby")]
    pub sortby: Option<SortType>,

    #[serde(default= "default_page")]
    pub page: Option<u32>,

    #[serde(default= "default_results_per_page")]
    pub results_per_page: Option<u16>,

}

/// Represents the parameters of a given relational search
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")] 
pub struct RelationalSearchParameters {

    pub hops: u8,
    pub root: String,

    #[serde(default = "default_query_relational")]
    pub query: Option<String>,

    #[serde(default = "default_results_per_page")]
    pub max_results: Option<u16>,
}

/// Represents a wikipedia article
#[derive(Serialize)]
pub struct Document{

    pub title: String,
    pub score: f64,

    #[serde(rename = "abstract")]
    pub article_abstract: String,
}

/// Represents a relation between two articles 
/// where source is the origin of a link 
/// and destination is the destination of the link
#[derive(Serialize)]
pub struct Relation{

    pub source: String,
    pub destination: String,
}

/// Represents a collection of documents and relations 
#[derive(Serialize)]
pub struct RelationSearchOutput{
    pub documents: Vec<Document>,
    pub relations: Vec<Relation>, 
}

fn default_sortby() -> Option<SortType> {Option::from(SortType::Relevance)}
fn default_page() -> Option<u32> {Option::from(1)}
fn default_results_per_page() -> Option<u16> {Option::from(20)}
fn default_query_relational() -> Option<String> {Option::from("".to_string())}
