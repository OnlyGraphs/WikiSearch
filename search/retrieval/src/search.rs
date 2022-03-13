use crate::tfidf_query;

use index::{
    index::Index,
    index_structs::{Posting}, PosRange,
};

use itertools::Itertools;

use parser::{ast::{Query}, UnaryOp, BinaryOp};
use parser::errors::{QueryError, QueryErrorKind};
use preprocessor::{Preprocessor, ProcessingOptions};

use std::{collections::{HashMap, VecDeque}, iter::empty};
use utils::utils::merge;

#[derive(Debug, PartialEq, PartialOrd)]
pub struct ScoredDocument {
    pub score: f64,
    pub doc_id: u32,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct ScoredRelationDocument {
    pub score: f64,
    pub doc_id: u32,
    pub hops: u8,
}

pub fn preprocess_query(query: &mut Query) -> Result<(), QueryError> {
    // first pass, preprocess
    let ref opts = ProcessingOptions::default();

    match *query {
        Query::RelationQuery { ref mut sub, .. } => {
            drop(sub.as_mut().map(|c| Some(preprocess_query(c).ok()?)))
        }
        Query::StructureQuery { ref mut sub, .. } => preprocess_query(sub)?,
        Query::UnaryQuery { ref mut sub, .. } => preprocess_query(sub)?,
        Query::BinaryQuery {
            ref mut lhs,
            ref mut rhs,
            ..
        } => {
            preprocess_query(lhs)?;
            preprocess_query(rhs)?;
        }
        Query::PhraseQuery { ref mut tks } => {
            *tks = tks
                .into_iter()
                .flat_map(|c| Preprocessor::process(opts, c.to_string()))
                .filter(|w| !w.trim().is_empty())
                .collect()
        }
        Query::FreetextQuery { ref mut tokens } => {
            *tokens = tokens
                .into_iter()
                .flat_map(|c| Preprocessor::process(opts, c.to_string()))
                .filter(|w| !w.trim().is_empty())
                .collect()
        }
        Query::DistanceQuery {
            ref mut lhs,
            ref mut rhs,
            ..
        } => {
            *lhs = Preprocessor::process(opts, lhs.clone())
                .into_iter()
                .next()
                .ok_or(QueryError {
                    kind: QueryErrorKind::InvalidSyntax,
                    msg: "Distance query requires at least one individual word on each side"
                        .to_string(),
                    pos: lhs.to_string(),
                })?;
            *rhs = Preprocessor::process(opts, rhs.clone())
                .into_iter()
                .next()
                .ok_or(QueryError {
                    kind: QueryErrorKind::InvalidSyntax,
                    msg: "Distance query requires at least one individual word on each side"
                        .to_string(),
                    pos: rhs.to_string(),
                })?;
        }
        Query::WildcardQuery {
            ref mut prefix,
            ref mut postfix,
        } => {
            *prefix = prefix.to_lowercase(); // needs a more thorough look
            *postfix = postfix.to_lowercase();
        }
    };

    Ok(())
}

pub struct PostingIterator<'a> {
    wrapped : Box<dyn Iterator<Item = Posting> + 'a>
}

impl <'a>PostingIterator<'a>{
    pub fn new<T : Iterator<Item= Posting> + 'a>(o: T) -> Self {
        Self {
            wrapped: Box::new(o) 
        }
    }

    pub fn rewrap<T : Iterator<Item = Posting> + 'a>(mut me : Self ,o : T) -> Self{
        me.wrapped = Box::new(o);
        me
    }
}

impl Iterator for PostingIterator<'_> {
    type Item = Posting;

    fn next(&mut self) -> Option<Self::Item> {
        self.wrapped.next()
    }
        
}


//TODO: get rid of posting copying, do stuff by reference, + batch postings list in case we run out of memory
pub fn execute_query<'a>(query: &'a Box<Query>, index: &'a Index) -> PostingIterator<'a> {

    match **query {
        Query::DistanceQuery {
            ref dst,
            ref lhs,
            ref rhs,
        } =>  {
            let lhs = index.get_postings(lhs).map(|v| v.lock().get().unwrap().postings.into_iter().collect::<Vec<Posting>>());
            let rhs = index.get_postings(rhs).map(|v| v.lock().get().unwrap().postings.into_iter().collect::<Vec<Posting>>());

            if lhs.is_none() || rhs.is_none(){
                return PostingIterator{
                    wrapped: Box::new(empty::<Posting>()),
                }
            }
        
            PostingIterator::new(
                DistanceMergeIterator::new(
                    *dst,
                    Box::new(lhs.unwrap().into_iter()),
                    Box::new(rhs.unwrap().into_iter())
                ),
            )
        },
        Query::RelationQuery {
            root: id,
            ref hops,
            ref sub,
        } => {
            let mut subset = HashMap::default();
            get_docs_within_hops(id, *hops, &mut subset, index);

            match sub {
                Some(q) => {
                    PostingIterator::new(execute_query(q, index).into_iter().filter(move |v| {
                        subset.contains_key(&v.document_id)
                    }))
                },
                None => {
                    let mut o = subset.into_iter().map(|(k,v)|{
                        Posting{
                            document_id: k,
                            position: v as u32,
                        }
                    }).collect::<Vec<Posting>>();
                    o.sort();

                    PostingIterator::new(o.into_iter())
                },
            }
        },
        Query::WildcardQuery {..} => todo!(),// TODO: needs index support
        Query::StructureQuery { ref elem, ref sub } => 
            PostingIterator::new(
                execute_query(sub, index)
                .filter(
                    |c| match index.get_extent_for(elem.clone().into(), &c.document_id) {
                        Some(PosRange { start_pos, end_pos }) => {
                            c.position >= *start_pos && c.position < *end_pos
                        }
                        None => false,
                    },
                )
            ),
        Query::PhraseQuery { ref tks } => {
            let  init  = PostingIterator::new(empty::<Posting>());

            tks.iter().tuple_windows().map(|(a,b)| {
                let l = index.get_postings(a).map(|v| {
                    v.lock().get().unwrap().postings.into_iter().collect::<Vec<Posting>>()
                
                });
                let r = index.get_postings(b).map(|v| {
                    v.lock().get().unwrap().postings.into_iter().collect::<Vec<Posting>>()
                });
                (l,r)
            }).enumerate()
                .fold(init,
                    |a, (i,(l,r))| {
                        let lhs: Box<dyn Iterator<Item = Posting>> = Box::new(l.unwrap_or_default().into_iter());
                        let rhs: Box<dyn Iterator<Item = Posting>> = Box::new(r.unwrap_or_default().into_iter());

                        let curr = DistanceMergeIterator::new(1, lhs, rhs);

                        if i != 0 {
                            PostingIterator::new(
                                DistanceMergeIterator::new(i as u32, Box::new(a), Box::new(curr))
                            )
                        } else {
                            PostingIterator::rewrap(a,curr)
                        }
                    },
                )
        },

        Query::UnaryQuery { ref op, ref sub } => match op {
            UnaryOp::Not => execute_query(sub, index) // soft not 
        },
        Query::BinaryQuery {
            ref op,
            ref lhs,
            ref rhs,
        } => {
            let sub_l = execute_query(lhs, index); 
            let sub_r = execute_query(rhs, index); 
            match op {
                BinaryOp::And => 
                    PostingIterator::new(IntersectionMergeIterator::new(
                        Box::new(sub_l),
                        Box::new(sub_r)
                    )),
                BinaryOp::Or =>
                    PostingIterator::new(UnionMergeIterator::new(
                        Box::new(sub_l),
                        Box::new(sub_r)
                    ))
            }
        },
        Query::FreetextQuery { ref tokens } => {
            let init  = PostingIterator::new(empty::<Posting>());


            tokens.iter()
                .filter_map(|v| index.get_postings(v))
                .fold(init,|a,iter| {
                PostingIterator::new(
                    UnionMergeIterator::new(
                        Box::new(a),
                        Box::new(iter.lock().get().unwrap().postings.into_iter().collect::<Vec<Posting>>().into_iter())
                    )
                )
            })
        },
        _ => todo!(),
    }
}

/// own endpoint for relational query, scoring for it should happen here (i.e. Page Rank)
pub fn execute_relational_query<'a>(query: &'a Box<Query>, index: &'a Index) -> Vec<ScoredRelationDocument> {

        if let Query::RelationQuery{root, hops, sub} = &**query{
            let mut subset = HashMap::default();
            get_docs_within_hops(*root, *hops, &mut subset, index);

            match sub {
                Some(v) => {
                    execute_query(&v, index)
                            .filter_map(move |c| {
                                if subset.contains_key(&c.document_id){
                                    Some(ScoredRelationDocument{
                                        score: *index.page_rank.get(&c.document_id).unwrap_or(&0.0), // PAGE RANK
                                        doc_id: c.document_id,
                                        hops: *subset.get(&c.document_id).unwrap()
                                    })
                                } else {
                                    None
                                }
                            }).collect()
                }
                None => {
                    subset
                        .into_iter()
                        .map(move |(id,hops)| ScoredRelationDocument {
                            score: *index.page_rank.get(&id).unwrap_or(&0.0), // PAGE RANK
                            doc_id: id,
                            hops: hops, // magic number, choose whatever you want
                        })
                        .collect()
                }
            }
        } else {
            Vec::default()
        }
}

/// finds documents within given hops off the root, also stores the number of hops from the root
pub fn get_docs_within_hops(docid: u32, hops: u8, out: &mut HashMap<u32,u8>, index: &Index) {
    let mut queue = VecDeque::default();
    let mut depth_increasing_nodes = VecDeque::default();

    queue.push_back(docid);
    depth_increasing_nodes.push_back(docid);

    let mut curr_hops = 0;
    loop {
        let top = queue.pop_front();

        if let Some(top) = top {
            out.insert(top, curr_hops);

            if let Some(v) = depth_increasing_nodes.front(){
                if *v == top{
                    curr_hops += 1;
                    depth_increasing_nodes.pop_front();
                }
            }
            
            if curr_hops == hops + 1{
                return;
            }

            let out_l = index.get_links(top);
            let in_l = index.get_incoming_links(top);
            let all_l = merge(in_l, out_l);

            let mut added = false;
            all_l.iter().for_each(|v| {
                if !out.contains_key(v) {
                    queue.push_back(*v);
                    added = true;
                }
            });

            if let Some(v) = queue.back() {
                if added {
                    depth_increasing_nodes.push_back(*v);
                }
            }

        } else {
            return;
        }
    }
}




/// scores all queries apart from the relational query which passes through its own endpoint
pub fn score_query(
    query: &Box<Query>,
    index: &Index,
    postings: &mut Vec<Posting>,
) -> Vec<ScoredDocument> {
    postings.dedup_by_key(|v| v.document_id);
    let mut scored_documents = Vec::default();

    for post in postings {

        let mut page_rank = 0.0;
        let pr = index.page_rank.get(&post.document_id);
        match pr {
            Some(v) => page_rank = *v,
            _ => page_rank = 0.0
        };

        scored_documents.push(ScoredDocument {
            doc_id: post.document_id,
            score: tfidf_query(post.document_id, query, index)*0.9+page_rank*0.1,
        });
    }
    return scored_documents;
}



#[derive(Clone,Copy, Debug, Eq, PartialEq)]
pub enum SkipSide {
    Left,
    Right,
}

// ------
// Union 
// ------

enum UnionMergeState {
    None,
    Right, 
    Left
}

pub struct UnionMergeIterator<'a> {
    left_iter:  Box<dyn Iterator<Item = Posting> + 'a>,
    right_iter: Box<dyn Iterator<Item = Posting> + 'a>,
    state: UnionMergeState,
    last : (Option<Posting>, Option<Posting>)

} 
impl <'a>UnionMergeIterator<'a> {
    pub fn new(l: Box<dyn Iterator<Item = Posting> + 'a>,
       r: Box<dyn Iterator<Item = Posting> + 'a>) -> Self{
        Self {
            left_iter: l,
            right_iter: r,
            state: UnionMergeState::None,
            last: (None,None),
            
        }
    }
}

impl <'a>Iterator for UnionMergeIterator<'a>{
    type Item = Posting;

    fn next(&mut self) -> Option<Self::Item> {        
        self.last = match self.state {
            UnionMergeState::Left => { // last time left side was 'get', advance it
                self.state = UnionMergeState::Left;
                (self.left_iter.next(), self.last.1)
            },
            UnionMergeState::Right=> { // last time right side was 'get', advance it 
                self.state = UnionMergeState::Right;    
                (self.last.0, self.right_iter.next()) 
            },
            UnionMergeState::None => {
                (self.left_iter.next(),self.right_iter.next())
            },
        };

        match self.last{
            (None, None)    => self.state = UnionMergeState::None, // loop around
            (None, Some(_)) => self.state = UnionMergeState::Right,// pick right
            (Some(_), None) => self.state = UnionMergeState::Left, // pick left
            (Some(l), Some(r)) if l <= r  => self.state = UnionMergeState::Left,
            _ => self.state = UnionMergeState::Right, 
        }
        
        match self.state {
                    UnionMergeState::Left => self.last.0,
                    UnionMergeState::Right=> self.last.1,
                    UnionMergeState::None => None,  
        }
    }
}

// ------------
// Intersection
// ------------

pub enum IntersectionMergeState {
    None,
    LeftThenRightStart,
    LeftThenRightFinish,
    RightThenLeftStart,
    RightThenLeftFinish,
}

pub struct IntersectionMergeIterator<'a> {
    pub left_iter:  Box<dyn Iterator<Item = Posting> + 'a>,
    pub right_iter: Box<dyn Iterator<Item = Posting> + 'a>,
    pub state: IntersectionMergeState,
    pub curr: (Option<Posting>, Option<Posting>)

} 

impl <'a> IntersectionMergeIterator<'a> {
   pub fn new(
       l:Box<dyn Iterator<Item = Posting> + 'a>,
       r:Box<dyn Iterator<Item = Posting> + 'a>,
    ) -> Self {
        Self {
            left_iter: l,
            right_iter: r,
            state: IntersectionMergeState::None,
            curr: (None,None),
        }
    }


    fn advance(&mut self) {
        self.curr = match self.state {
            IntersectionMergeState::RightThenLeftFinish => { // last time left side was 'get', advance it
                (self.left_iter.next(), self.curr.1)
            },
            IntersectionMergeState::LeftThenRightFinish => { // last time right side was 'get', advance it 
                (self.curr.0, self.right_iter.next()) 
            },
            IntersectionMergeState::None => {
                (self.left_iter.next(),self.right_iter.next())
            },
            IntersectionMergeState::LeftThenRightStart => { // completed left, need right
                self.state = IntersectionMergeState::LeftThenRightFinish;
                self.curr = (self.left_iter.next(),self.curr.1);
                return;
            }, 
            IntersectionMergeState::RightThenLeftStart => { // completed right, need left
                self.state = IntersectionMergeState::RightThenLeftFinish;
                self.curr = (self.curr.0,self.right_iter.next());
                return;
            }  
        };

        let mut skip_side;
        loop {
            match self.curr {
                (None, None) => {self.state = IntersectionMergeState::None; break;}, // loop around
                (None, Some(_)) => {skip_side = SkipSide::Right},
                (Some(_), None) => {skip_side = SkipSide::Left},
                (Some(l), Some(r)) => {
                    if l.document_id == r.document_id{
                        if l <= r {
                            self.state= IntersectionMergeState::LeftThenRightStart; 
                            break;
                        } else {
                            self.state= IntersectionMergeState::RightThenLeftStart; 
                            break;
                        }
                    } else {
                        if l <= r {
                            skip_side = SkipSide::Left;
                        } else {
                            skip_side = SkipSide::Right;
                        }
                    }
                
                }
            }

            self.curr = match skip_side {
                SkipSide::Left =>  (self.left_iter.next(),self.curr.1),
                SkipSide::Right => (self.curr.0, self.right_iter.next()),
            };
        }

    }

    fn get(&self) -> Option<Posting> {
        match self.state {
            IntersectionMergeState::LeftThenRightStart | IntersectionMergeState::RightThenLeftFinish => self.curr.0,
            IntersectionMergeState::LeftThenRightFinish | IntersectionMergeState::RightThenLeftStart => self.curr.1, 
            IntersectionMergeState::None => None,
        }
    }
} 

impl <'a>Iterator for IntersectionMergeIterator<'a>{
    type Item = Posting;

    fn next(&mut self) -> Option<Self::Item> {
        self.advance();
        self.get()
    }
}

// ------
// Difference
// ------

pub enum DifferenceMergeState {
    None,
    Left,
    LeftThenSkipRightStart,
    LeftThenSkipRightFinish
}

pub struct DifferenceMergeIterator<'a> {
    left_iter:  Box<dyn Iterator<Item = Posting> + 'a>,
    right_iter: Box<dyn Iterator<Item = Posting> + 'a>,
    state: DifferenceMergeState,
    highest_doc_r: i32,
    curr: (Option<Posting>, Option<Posting>)
} 

impl <'a> DifferenceMergeIterator<'a> {
    pub fn new(
        l:Box<dyn Iterator<Item = Posting> + 'a>,
        r:Box<dyn Iterator<Item = Posting> + 'a>,
    ) -> Self {
        Self {
            left_iter: l,
            right_iter: r,
            state: DifferenceMergeState::None,
            highest_doc_r: -1,
            curr: (None,None),
        }
    }


    fn advance(&mut self) {
        self.curr = match self.state {
            DifferenceMergeState::Left => { // last time left side was 'get', advance it
                (self.left_iter.next(), self.curr.1)
            },
            DifferenceMergeState::LeftThenSkipRightFinish => { // last time right side was 'get', advance it 
                (self.curr.0, self.right_iter.next()) 
            },
            DifferenceMergeState::None => {
                (self.left_iter.next(),self.right_iter.next())
            },
            DifferenceMergeState::LeftThenSkipRightStart => {
                self.state = DifferenceMergeState::LeftThenSkipRightFinish;
                (self.left_iter.next(),self.right_iter.next())
            },      
        };
    
        let mut skip_side;
        loop {
            match self.curr{
                (None, None) => {
                    self.state = DifferenceMergeState::None; 
                    break;
                }, 
                (None, Some(r)) => {
                    skip_side = SkipSide::Right;
                    self.highest_doc_r = r.document_id as i32;
                },
                (Some(l), None) => {
                    if l.document_id != self.highest_doc_r as u32 {
                        self.state = DifferenceMergeState::Left; 
                        break;
                    } else {
                        skip_side = SkipSide::Left;
                    }
                }, 
                (Some(l), Some(r)) => {
                    self.highest_doc_r = r.document_id as i32;

                    if l.document_id < r.document_id{
                        self.state= DifferenceMergeState::Left; 
                        break;
                    } else if l <= r {
                        skip_side = SkipSide::Left
                    } else if l > r {
                        skip_side = SkipSide::Right
                    } else {
                        self.state= DifferenceMergeState::LeftThenSkipRightStart; 
                        break;
                    }
                },
            };

            self.curr = match skip_side {
                SkipSide::Left =>  (self.left_iter.next(),self.curr.1),
                SkipSide::Right => (self.curr.0, self.right_iter.next()),
            };
        }
    }

    fn get(&self) -> Option<Posting> {
        match self.state {
            DifferenceMergeState::Left | DifferenceMergeState::LeftThenSkipRightStart => self.curr.0,
            DifferenceMergeState::None => None,  
            _ => panic!() // should not receive any skips here          
        }
    }
} 

impl <'a>Iterator for DifferenceMergeIterator<'a>{
    type Item = Posting;


    fn next(&mut self) -> Option<Self::Item> {
        self.advance();
        self.get()
    }
}

// --------
// Distance
// --------

#[derive(PartialEq, Eq)]
enum DistanceMergeState {
    None,
    Right,
    LeftThenRightStart,
    LeftThenRightFinish
}

pub struct DistanceMergeIterator<'a> {
    left_iter:  Box<dyn Iterator<Item = Posting> + 'a>,
    right_iter: Box<dyn Iterator<Item = Posting> + 'a>,
    state: DistanceMergeState,
    dst: u32,
    in_streak: bool,
    streak_start_pos : u32,
    curr: (Option<Posting>, Option<Posting>)
} 

impl <'a> DistanceMergeIterator<'a> {
    pub fn new(
        dst: u32,
        l:Box<dyn Iterator<Item = Posting> + 'a>,
        r:Box<dyn Iterator<Item = Posting> + 'a>,
    ) -> Self {
        Self {
            left_iter: l,
            right_iter: r,
            state: DistanceMergeState::None,
            dst: dst,
            in_streak: false,
            streak_start_pos: 0,
            curr: (None,None),            
        }
    }

    fn advance(&mut self) {
        self.curr = match self.state {
            DistanceMergeState::Right | DistanceMergeState::LeftThenRightFinish  => { // last time right side was 'get', advance it 
                (self.curr.0, self.right_iter.next()) 
            },
            DistanceMergeState::None => {
                (self.left_iter.next(),self.right_iter.next())
            },
            DistanceMergeState::LeftThenRightStart => { // completed left, need right
                self.state = DistanceMergeState::LeftThenRightFinish;
                self.curr = (self.left_iter.next(),self.curr.1);
                return;
            },   
        };

        let mut skip_side;
        loop {
            match self.curr{
                (Some(l), Some(r)) => {
                        if l.document_id == r.document_id {
                            // compare to buffered l
                            if self.in_streak && r.position.overflowing_sub(self.streak_start_pos).0 <= self.dst  {
                                self.state = DistanceMergeState::Right;
                                break;
                            // compared to fresh l
                            } else if r.position.overflowing_sub(l.position).0 <= self.dst{
                                self.state = DistanceMergeState::LeftThenRightStart;
                                self.in_streak = true;
                                self.streak_start_pos = l.document_id;
                                break;   
                            } 
                        }
                        
                        if l < r {
                            self.in_streak = false;
                            skip_side = SkipSide::Left
                        } else {
                            self.in_streak = false;
                            skip_side = SkipSide::Right
                        }

                    },
                (None, Some(r)) => {
                    if self.in_streak && r.position.overflowing_sub(self.streak_start_pos).0 <= self.dst {
                        self.state = DistanceMergeState::Right;
                        break;
                    } else {
                        self.state = DistanceMergeState::None;
                        break;
                    }
                },
                (_, _) => {
                    self.state = DistanceMergeState::None; 
                    break; 
                },             
            };

            self.curr = match skip_side {
                SkipSide::Left =>  (self.left_iter.next(),self.curr.1),
                SkipSide::Right => (self.curr.0, self.right_iter.next()),
            };
        }
    }
    

    fn get(&self) -> Option<Posting> {
        match self.state {
            DistanceMergeState::LeftThenRightStart => self.curr.0,
            DistanceMergeState::Right | 
            DistanceMergeState::LeftThenRightFinish  => self.curr.1,
            DistanceMergeState::None => None,  
        }
    }

} 

impl <'a>Iterator for DistanceMergeIterator<'a>{
    type Item = Posting;


    fn next(&mut self) -> Option<Self::Item> {
        self.advance();
        self.get()
    }
}