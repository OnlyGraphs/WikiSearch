use crate::index::errors::IndexError;
use crate::index::index::Index;
use crate::index::index_structs::Posting;
use crate::parser::ast::{BinaryOp, Query, UnaryOp};
use itertools::Itertools;
use std::cmp::Ordering;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ScoredPosting {
    score: u32,
    posting: Posting,
}

//TODO: get rid of posting copying, do stuff by reference, + batch postings list in case we run out of memory
pub fn execute_query(query: Box<Query>, index: &Box<dyn Index>) -> Vec<Posting> {
    println!("{:?}", query);
    match *query {
        Query::RelationQuery { root, hops, sub } => Vec::default(), // TODO: needs graph crawling
        Query::WildcardQuery { prefix, postfix } => Vec::default(), // TODO: needs index support
        Query::StructureQuery { elem, sub } => Vec::default(),      // TODO: it's 6 am
        Query::PhraseQuery { tks } => {
            tks.iter()
                .tuple_windows()
                .fold(Vec::default(), |a, (prev, current)| {
                    println!("TOKENS: {:?}", (prev, current));
                    linear_merge(
                        a,
                        linear_merge(
                            index
                                .as_ref()
                                .get_postings(prev)
                                .unwrap_or_default()
                                .to_vec(),
                            index
                                .as_ref()
                                .get_postings(current)
                                .unwrap_or_default()
                                .to_vec(),
                            MergeType::Distance(1),
                        ),
                        MergeType::Intersection, // left merge
                    )
                })
        }
        Query::DistanceQuery { dst, lhs, rhs } => linear_merge(
            index
                .as_ref()
                .get_postings(&lhs)
                .unwrap_or_default()
                .to_vec(),
            index
                .as_ref()
                .get_postings(&rhs)
                .unwrap_or_default()
                .to_vec(),
            MergeType::Distance(dst),
        ),
        Query::UnaryQuery { op, sub } => match op {
            UnaryOp::Not => linear_merge(
                index.get_all_postings(),
                execute_query(sub, index),
                MergeType::Subtraction,
            ),
        },
        Query::BinaryQuery { op, lhs, rhs } => match op {
            BinaryOp::And => linear_merge(
                execute_query(lhs, index),
                execute_query(rhs, index),
                MergeType::Intersection,
            ),
            BinaryOp::Or => linear_merge(
                execute_query(lhs, index),
                execute_query(rhs, index),
                MergeType::Union(MergeIdentity::FullMerge),
            ),
        },
        Query::FreetextQuery { tokens } => tokens.iter().fold(Vec::default(), |a, t| {
            linear_merge(
                a,
                index.as_ref().get_postings(t).unwrap_or_default().to_vec(),
                MergeType::Union(MergeIdentity::FullMerge),
            )
        }),
    }
}

pub fn rank_query_results(query: Box<Query>, result: &Vec<Posting>) -> Vec<ScoredPosting> {
    Vec::default()
}

#[derive(Eq, PartialEq)]
pub enum MergeType {
    Union(MergeIdentity), // a + b
    Intersection,         // a.docid == b.docid
    Subtraction,          // a.docid != b.docid
    Distance(u32),        // a.docid == b.docid && b.position - a.position = dst
}
#[derive(Eq, PartialEq)]
pub enum MergeIdentity {
    LeftMerge,
    RightMerge,
    FullMerge,
}

fn linear_merge(a: Vec<Posting>, b: Vec<Posting>, operation: MergeType) -> Vec<Posting> {
    println!("----- {:?}:{:?}", a, b);
    let mut iter_left = a.iter();
    let mut iter_right = b.iter();
    let mut curr_items = (iter_left.next(), iter_right.next());
    let mut out = Vec::new();
    loop {
        println!("{:?}, {:?}", curr_items, out);
        match curr_items {
            (Some(l), None) => {
                if let MergeType::Union(..) = operation {
                    out.push(*l)
                }
                curr_items.0 = iter_left.next();
            }
            (None, Some(r)) => {
                if let MergeType::Union(..) = operation {
                    out.push(*r)
                }
                curr_items.1 = iter_right.next();
            }
            (Some(l), Some(r)) => {
                if operation == MergeType::Subtraction && l.document_id < r.document_id {
                    out.push(*l);
                }

                if l.document_id == r.document_id {
                    match operation {
                        MergeType::Distance(dst) => {
                            if r.position.overflowing_sub(l.position).0 <= dst {
                                if l < r {
                                    out.push(*l);
                                    out.push(*r)
                                } else {
                                    out.push(*r);
                                    out.push(*l)
                                }
                            }
                        }
                        MergeType::Union(MergeIdentity::FullMerge) => {
                            if l < r {
                                out.push(*l);
                                out.push(*r)
                            } else {
                                out.push(*r);
                                out.push(*l)
                            }
                        }
                        MergeType::Union(MergeIdentity::LeftMerge) => out.push(*l),
                        MergeType::Union(MergeIdentity::RightMerge) => out.push(*r),
                        MergeType::Intersection => {
                            if l < r {
                                out.push(*l);
                                out.push(*r)
                            } else {
                                out.push(*r);
                                out.push(*l)
                            }
                        }
                        MergeType::Subtraction => (),
                    }

                    if l.position == r.position {
                        curr_items.0 = iter_left.next();
                        curr_items.1 = iter_right.next();
                    } else if l.position < r.position {
                        curr_items.0 = iter_left.next();
                    } else {
                        curr_items.1 = iter_right.next();
                    }
                } else if l.document_id < r.document_id {
                    curr_items.0 = iter_left.next();
                } else {
                    curr_items.1 = iter_right.next();
                }
            }
            (None, None) => break,
        };
    }

    return out;
}
