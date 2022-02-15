use std::sync::Arc;
use std::sync::RwLock;
use crate::index::index_structs::Posting;
use crate::Index;
use crate::parser::ast::{Query};
use crate::index::errors::IndexError;

fn execute_query(query: Box<Query>, index: Arc<RwLock<Box<dyn Index>>>) -> Result<Vec<Posting>, IndexError>{
    
    match *query{
        Query::BinaryQuery{op, lhs, rhs} => Ok(Vec::default()),
        Query::UnaryQuery{op, sub} => Ok(Vec::default()),
        Query::PhraseQuery{tks} => Ok(Vec::default()),
        Query::DistanceQuery{dst, lhs, rhs} => Ok(Vec::default()),
        Query::StructureQuery{elem, sub} => Ok(Vec::default()),
        Query::RelationQuery{root, hops, sub} => Ok(Vec::default()),
        Query::WildcardQuery{prefix, postfix} => Ok(Vec::default()),
        Query::FreetextQuery{tokens} => Ok(Vec::default()),
    }
}

fn rank_query_results(query: Box<Query>, result : &Vec<Posting>) -> Vec<(u32,u32)>{
    Vec::default()
}

