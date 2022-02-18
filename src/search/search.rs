use crate::index::index::Index;
use crate::index::index_structs::Posting;
use crate::index_structs::PosRange;
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
    match *query {
        Query::RelationQuery { root, hops, sub } => Vec::default(), // TODO: needs graph crawling
        Query::WildcardQuery { prefix, postfix } => Vec::default(), // TODO: needs index support
        Query::StructureQuery { elem, sub } => execute_query(sub, index)
            .into_iter()
            .filter(
                |c| match index.get_extent_for(elem.clone().into(), &c.document_id) {
                    Some(PosRange { start_pos, end_pos }) => {
                        c.position >= *start_pos && c.position < *end_pos
                    }
                    None => false,
                },
            )
            .collect(),
        Query::PhraseQuery { tks } => tks.iter().tuple_windows().enumerate().fold(
            Vec::default(),
            |a, (i, (prev, current))| {
                let curr = distance_merge(
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
                    1,
                );
                if i != 0 {
                    return distance_merge(a, curr, i as u32);
                } else {
                    return curr;
                }
            },
        ),
        Query::DistanceQuery { dst, lhs, rhs } => distance_merge(
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
            dst,
        ),
        Query::UnaryQuery { op, sub } => match op {
            UnaryOp::Not => difference_merge(index.get_all_postings(), execute_query(sub, index)),
        },
        Query::BinaryQuery { op, lhs, rhs } => match op {
            BinaryOp::And => {
                intersection_merge(execute_query(lhs, index), execute_query(rhs, index))
            }
            BinaryOp::Or => union_merge(execute_query(lhs, index), execute_query(rhs, index)),
        },
        Query::FreetextQuery { tokens } => tokens.iter().fold(Vec::default(), |a, t| {
            union_merge(
                a,
                index.as_ref().get_postings(t).unwrap_or_default().to_vec(),
            )
        }),
    }
}

pub fn score_query(query: Box<Query>, index: &Box<dyn Index>, postings: &Vec<Posting>) -> Vec<ScoredPosting> {
    Vec::default()
}


fn distance_merge(a: Vec<Posting>, b: Vec<Posting>, dst: u32) -> Vec<Posting> {
    let mut iter_left = a.iter();
    let mut iter_right = b.iter();
    let mut curr_items = (iter_left.next(), iter_right.next());
    let mut out = Vec::new();

    loop {
        let (l, r) = match curr_items {
            (Some(_), None) => return out,
            (None, Some(_)) => return out,
            (Some(l), Some(r)) => (l, r),
            (None, None) => break,
        };

        if l.document_id == r.document_id {
            if r.position.overflowing_sub(l.position).0 <= dst {
                out.push(*l); // only added at beginning
                out.push(*r);

                // consume all matches under distance, but not the first non match
                out.extend(iter_right.peeking_take_while(|c| {
                    c.document_id == l.document_id
                        && c.position.overflowing_sub(l.position).0 <= dst
                }));

                // move to next possible match or away to next doc
                curr_items.1 = iter_right.next();
                // curr_items.0 = iter_left.next();
            } else if l.position < r.position {
                curr_items.0 = iter_left.next();
            } else {
                curr_items.1 = iter_right.next();
            }
        } else {
            if l.document_id < r.document_id {
                curr_items.0 = iter_left.next();
            } else {
                curr_items.1 = iter_right.next();
            }
        }
    }

    return out;
}

fn union_merge(a: Vec<Posting>, b: Vec<Posting>) -> Vec<Posting> {
    let mut iter_left = a.iter();
    let mut iter_right = b.iter();
    let mut curr_items = (iter_left.next(), iter_right.next());
    let mut out = Vec::new();
    loop {
        match curr_items {
            (Some(l), None) => {
                out.push(*l);
                curr_items.0 = iter_left.next();
            }
            (None, Some(r)) => {
                out.push(*r);
                curr_items.1 = iter_right.next();
            }
            (Some(l), Some(r)) => {
                if l.document_id == r.document_id {
                    if l.position == r.position {
                        out.push(*l);
                        out.push(*r);

                        curr_items.0 = iter_left.next();
                        curr_items.1 = iter_right.next();
                    } else if l.position < r.position {
                        out.push(*l);
                        curr_items.0 = iter_left.next();
                    } else {
                        out.push(*r);
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

fn intersection_merge(a: Vec<Posting>, b: Vec<Posting>) -> Vec<Posting> {
    let mut iter_left = a.iter();
    let mut iter_right = b.iter();
    let mut curr_items = (iter_left.next(), iter_right.next());
    let mut out = Vec::new();
    loop {
        match curr_items {
            (Some(_), None) => {
                curr_items.0 = iter_left.next();
            }
            (None, Some(_)) => {
                curr_items.1 = iter_right.next();
            }
            (Some(l), Some(r)) => {
                if l.document_id == r.document_id {
                    if l.position < r.position {
                        out.push(*l);
                        out.push(*r);
                    } else {
                        out.push(*r);
                        out.push(*l);
                    }
                    curr_items.0 = iter_left.next();
                    curr_items.1 = iter_right.next();
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

fn difference_merge(a: Vec<Posting>, b: Vec<Posting>) -> Vec<Posting> {
    let mut iter_left = a.iter();
    let mut iter_right = b.iter();
    let mut curr_items = (iter_left.next(), iter_right.next());
    let mut out = Vec::new();
    loop {
        match curr_items {
            (Some(_), None) => {
                curr_items.0 = iter_left.next();
            }
            (None, Some(_)) => {
                curr_items.1 = iter_right.next();
            }
            (Some(l), Some(r)) => {
                if l.document_id < r.document_id {
                    out.push(*l);
                }

                if l.document_id == r.document_id {
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
