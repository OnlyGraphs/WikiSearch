use std::sync::Arc;
use std::sync::RwLock;
use crate::index::index_structs::Posting;
use crate::Index;
use crate::parser::ast::{Query};

fn execute_query(query: Box<Query>, index: Arc<RwLock<Box<dyn Index>>>) -> Vec<Posting>{
    match *query{
        Query::BinaryQuery{op, lhs, rhs} => Vec::default(),
        Query::UnaryQuery{op, sub} => Vec::default(),
        Query::PhraseQuery{tks} => Vec::default(),
        Query::DistanceQuery{dst, lhs, rhs} => Vec::default(),
        Query::StructureQuery{elem, sub} => Vec::default(),
        Query::RelationQuery{root, hops, sub} => Vec::default(),
        Query::WildcardQuery{prefix, postfix} => Vec::default(),
        Query::FreetextQuery{tokens} => Vec::default(),
    }
}

fn rank_query_results(query: Box<Query>, result : &Vec<Posting>) -> Vec<(u32,u32)>{
    Vec::default()
}

