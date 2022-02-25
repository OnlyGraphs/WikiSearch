use chrono::NaiveDateTime;

use std::collections::HashMap;

pub const DATE_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// stores an appearance of a token in an article
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy)] //TODO: get rid of Copy and correct parts of program which use it
pub struct Posting {
    pub document_id: u32, //TODO: double check memory requirements, highest article word count etc
    pub position: u32,
}

#[derive(Debug, Eq, PartialEq, Default, Clone)]
pub struct PostingNode {
    pub postings: Vec<Posting>,
    pub df: u32,
    pub tf: HashMap<u32, u32>,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PosRange {
    pub start_pos: u32, //TODO: double check memory requirements, highest article word count etc
    pub end_pos: u32,
}

pub struct Infobox {
    pub itype: String,
    pub text: String,
}

pub struct Citation {
    pub text: String,
}

#[derive(Default)]
pub struct Document {
    pub doc_id: u32,
    pub title: String,
    pub categories: String,
    pub last_updated_date: String,
    pub namespace: i16,
    pub main_text: String,
    pub article_links: String,
    pub infoboxes: Vec<Infobox>,
    pub citations: Vec<Citation>,
}

#[derive(Debug)]
pub struct DocumentMetaData {
    pub title: String, //TODO: Implement another field with doc_id -> title and title -> doc_id
    pub last_updated_date: Option<NaiveDateTime>, //TODO: Change to DateTime type using chrono
    pub namespace: i16, //TODO: Could change this field to enum
}
