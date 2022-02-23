use std::collections::HashMap;
use crate::index::index::Index;
use crate::index::index_structs::Posting;
use crate::parser::ast::{Query, BinaryOp};

pub fn idf(df: f64, num_documents: u32) -> f64 {
    return ((num_documents as f64)/df).log10()
}

// Compute the tfidf for a single term
fn tfidf_term(term: &str, doc_id: u32, index: &Box<dyn Index>) -> f64 {
    let num_documents = index.get_number_of_documents();
    let tf = index.tf(term, doc_id);
    let idf_term = idf(index.df(term) as f64, num_documents);
    return (1.0 + (tf as f64).log10())*idf_term;
}

// compute the tfidf over a number of terms with regads to  certain document
pub fn tfidf_doc(terms: &Vec<String>, doc_id: u32, index: &Box<dyn Index>) -> f64 {
    let mut score = 0.0;
    for term in terms {
        score += tfidf_term(term, doc_id, index);
    }
    return score;
}

pub fn tfidf_query(posting: &Posting, query: Box<Query>, index: &Box<dyn Index>) -> f64 {

    match query {
        Query::FreetextQuery{tokens} => return tfidf_doc(&tokens, posting.document_id, index),
        Query::BinaryQuery{op, lhs, rhs} => return tfidf_query(posting, lhs, index) + tfidf_query(posting, rhs, index),
        Query::UnaryQuery{op, sub} => return -tfidf_query(posting, sub, index),
        Query::PhraseQuery{tks} => return tfidf_doc(&tks, posting.document_id,  index),
        Query::StructureQuery{elem, sub} => return tfidf_query(posting, sub, index),
        Query::RelationQuery{root, hops, sub} => {
            match *sub {
                Some(v) => return tfidf_query(posting, Box::new(v), index),
                _ => return 0.0,
            }
        },
        _ => return 0.0
    }
}