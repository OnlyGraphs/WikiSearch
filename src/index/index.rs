use crate::index::index_structs::*;
use either::{Either, Left};
use sqlx::Postgres;
use std::{collections::HashMap, hash::Hash};

#[derive(Debug)]
pub enum Domain {
    simple,
}

pub enum IndexEncoding {
    None,
    Delta_encoding,
    Elias_gamma_code,
}

//TODO: Interface to specify functions that should be shared among different types of indices created (Ternary Index Tree vs BasicIndex)
pub trait IndexInterface {
    fn add_posting(&mut self, token: String, docid: u32, word_pos: u32);
    fn add_document(
        &mut self,
        text: &str,
        doc_id: u32,
        categories: &str,
        article_links: &str,
        article_abstract: &str,
    );
    fn set_dump_id(new_dump_id: u32);
}

//TODO:
//Make sure you check for integer overflows. Or, implementing Delta encoding would mitigate any such problems.
#[derive(Debug)]
pub struct BasicIndex {
    pub dump_id: Option<u32>,
    pub document_metadata: HashMap<u32, DocumentMetaData>,
    //TODO: store tokens in a map, and store references in all others
    pub postings: HashMap<String, Vec<Posting>>,
    pub doc_freq: HashMap<String, u32>,
    pub term_freq: HashMap<String, HashMap<u32, u32>>, // tf(doc,term) -> frequency in document
    pub links: Either<HashMap<u32, Vec<String>>, HashMap<String, Vec<u32>>>, // List of tuples, where each element is: (Doc id, (Word_pos start, word_pos end))
    pub categories: HashMap<u32, Vec<String>>, //The name of category pages which a page links to  (eg. docid -> category1, category2).
    pub abstracts: HashMap<u32, String>,
    pub infoboxes: HashMap<u32, InfoBox>,
    pub citations: HashMap<u32, Citations>,
}

impl Default for BasicIndex {
    fn default() -> Self {
        BasicIndex {
            dump_id: None,
            postings: HashMap::new(),
            doc_freq: HashMap::new(),
            categories: HashMap::new(),
            abstracts: HashMap::new(),
            links: Left(HashMap::new()),
            term_freq: HashMap::new(),
            document_metadata: HashMap::new(),
            infoboxes: HashMap::new(),
            citations: HashMap::new(),
        }
    }
}

impl IndexInterface for BasicIndex {
    fn add_posting(&mut self, token: String, docid: u32, word_pos: u32) {
        self.postings
            .entry(token.clone())
            .or_insert(Vec::<Posting>::new())
            .push(Posting {
                document_id: docid,
                position: word_pos,
            });

        let freq_map: &mut HashMap<u32, u32> = self
            .term_freq
            .entry(token.clone())
            .or_insert(HashMap::new());
        *freq_map.entry(docid).or_insert(0) += 1;
    }

    fn add_document(
        &mut self,
        text: &str,
        doc_id: u32,
        categories: &str,
        article_links: &str,
        article_abstract: &str,
    ) {
        let mut word_pos = 0;
        for token in text.split(" ") {
            self.add_posting(token.to_string(), doc_id, word_pos);
            *self.doc_freq.entry(token.to_string()).or_insert(0) += 1;
            word_pos += 1;
        }

        let mut link_titles: Vec<String> = Vec::new();
        for link in article_links.split(",") {
            link_titles.push(link.trim().to_string());
        }

        self.links
            .as_mut()
            .expect_left("Index is not in buildable state")
            .insert(doc_id, link_titles);
        self.abstracts.insert(doc_id, article_abstract.to_string());
    }

    fn set_dump_id(new_dump_id: u32) {}
}
