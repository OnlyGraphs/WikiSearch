use crate::tfidf_query;

use index::{
    index::Index,
    index_structs::{Posting}, PosRange,
};

use itertools::Itertools;
use parser::{ast::{Query}, UnaryOp, BinaryOp};
use parser::errors::{QueryError, QueryErrorKind};
use preprocessor::{Preprocessor, ProcessingOptions};
use streaming_iterator::{StreamingIterator, convert, empty};

use std::{collections::HashSet};
use utils::utils::merge;

#[derive(Debug, PartialEq, PartialOrd)]
pub struct ScoredDocument {
    pub score: f64,
    pub doc_id: u32,
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
    wrapped : Box<dyn StreamingIterator<Item = Posting> + 'a>
}

impl <'a>PostingIterator<'a>{
    pub fn new<T : StreamingIterator<Item= Posting> + 'a>(o: T) -> Self {
        Self {
            wrapped: Box::new(o) 
        }
    }

    pub fn rewrap<T : StreamingIterator<Item = Posting> + 'a>(mut me : Self ,o : T) -> Self{
        me.wrapped = Box::new(o);
        me
    }
}

impl StreamingIterator for PostingIterator<'_> {
    type Item = Posting;

    #[inline(always)]
    fn advance(&mut self) {
        self.wrapped.advance()
    }

    #[inline(always)]
    fn get(&self) -> Option<&Self::Item> {
        self.wrapped.get()
    }
}


//TODO: get rid of posting copying, do stuff by reference, + batch postings list in case we run out of memory
pub fn execute_query<'a>(query: &'a Box<Query>, index: &'a Index) -> PostingIterator<'a> {

    match **query {
        Query::DistanceQuery {
            ref dst,
            ref lhs,
            ref rhs,
        } =>  PostingIterator::new(
                DistanceMergeStreamingIterator::new(
                    *dst,
                    Box::new(index.get_postings(lhs).unwrap()),
                    Box::new(index.get_postings(rhs).unwrap()),
                )
            ),
        Query::RelationQuery {
            root: id,
            ref hops,
            ref sub,
        } => {
            let mut subset = HashSet::default();
            get_docs_within_hops(id, *hops, &mut subset, index);

            match sub {
                Some(v) => {
                    return PostingIterator::new(
                                execute_query(v, index)
                                    .filter(move |c| subset.contains(&c.document_id))
                            )
                }
                None => {
                    let mut o: Vec<Posting> = subset
                        .into_iter()
                        .map(|c| Posting {
                            document_id: c,
                            position: 0,
                        })
                        .collect();
                    o.sort(); // TODO; hmm
                    return PostingIterator::new(convert(o));
                }
            };
        }
        Query::WildcardQuery {
            prefix: _,
            postfix: _,
        } => todo!(),// TODO: needs index support
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
                (index.get_postings(a).map(|v| {let a: Box<dyn StreamingIterator<Item = Posting>> = Box::new(v); a}).unwrap_or(Box::new(empty::<Posting>())),
                index.get_postings(b).map(|v| {let a: Box<dyn StreamingIterator<Item = Posting>> = Box::new(v); a}).unwrap_or(Box::new(empty::<Posting>())))
            }).enumerate()
                .fold(init,
                    |a, (i,(l,r))| {
                        let curr = DistanceMergeStreamingIterator::new(1, l, r);

                        if i != 0 {
                            PostingIterator::new(
                                DistanceMergeStreamingIterator::new(i as u32, Box::new(a), Box::new(curr))
                            )
                        } else {
                            PostingIterator::rewrap(a,curr)
                        }
                    },
                )
        },

        Query::UnaryQuery { ref op, ref sub } => match op {
            UnaryOp::Not => 
                PostingIterator::new(
                    DifferenceMergeStreamingIterator::new(
                        Box::new(index.get_all_postings()),
                        Box::new(execute_query(sub, index))
                    )
                ),
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
                    PostingIterator::new(IntersectionMergeStreamingIterator::new(
                        Box::new(sub_l),
                        Box::new(sub_r)
                    )),
                BinaryOp::Or =>
                    PostingIterator::new(UnionMergeStreamingIterator::new(
                        Box::new(sub_l),
                        Box::new(sub_r)
                    ))
            }
        },
        Query::FreetextQuery { ref tokens } => {
            let init  = PostingIterator::new(empty::<Posting>());

            tokens.iter().filter_map(|v| index.get_postings(v)).fold(init,|a,iter| {
                PostingIterator::new(
                    UnionMergeStreamingIterator::new(
                        Box::new(a),
                        Box::new(iter)
                    )
                )
            })
        },
        _ => todo!(),
    }
}




pub fn get_docs_within_hops(docid: u32, hops: u32, out: &mut HashSet<u32>, index: &Index) {
    out.insert(docid);

    if hops == 0 {
        return;
    }

    let out_l = index.get_links(docid);
    let in_l = index.get_incoming_links(docid);
    let all_l = merge(in_l, out_l);

    all_l.iter().for_each(|v| {
        if !out.contains(v) {
            out.insert(*v);
            get_docs_within_hops(*v, hops - 1, out, index);
        }
    })
}

pub fn score_query(
    query: &Box<Query>,
    index: &Index,
    postings: &mut Vec<Posting>,
) -> Vec<ScoredDocument> {
    postings.dedup_by_key(|v| v.document_id);
    let mut scored_documents = Vec::default();

    for post in postings {
        scored_documents.push(ScoredDocument {
            doc_id: post.document_id,
            score: tfidf_query(post.document_id, query, index),
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

pub struct UnionMergeStreamingIterator<'a> {
    left_iter:  Box<dyn StreamingIterator<Item = Posting> + 'a>,
    right_iter: Box<dyn StreamingIterator<Item = Posting> + 'a>,
    state: UnionMergeState,

} 
impl <'a>UnionMergeStreamingIterator<'a> {
    pub fn new(l: Box<dyn StreamingIterator<Item = Posting> + 'a>,
       r: Box<dyn StreamingIterator<Item = Posting> + 'a>) -> Self{
        Self {
            left_iter: l,
            right_iter: r,
            state: UnionMergeState::None,
        }
    }
}

impl <'a>StreamingIterator for UnionMergeStreamingIterator<'a>{
    type Item = Posting;

    fn advance(&mut self) {
        let items = match self.state {
            UnionMergeState::Left => { // last time left side was 'get', advance it
                self.state = UnionMergeState::Left;
                (self.left_iter.next(), self.right_iter.get())
            },
            UnionMergeState::Right=> { // last time right side was 'get', advance it 
                self.state = UnionMergeState::Right;    
                (self.left_iter.get(), self.right_iter.next()) 
            },
            UnionMergeState::None => {
                (self.left_iter.next(),self.right_iter.next())
            },
        };

        match items{
            (None, None)    => self.state = UnionMergeState::None, // loop around
            (None, Some(_)) => self.state = UnionMergeState::Right,// pick right
            (Some(_), None) => self.state = UnionMergeState::Left, // pick left
            (Some(l), Some(r)) if l <= r  => self.state = UnionMergeState::Left,
            _ => self.state = UnionMergeState::Right, 
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        match self.state {
                    UnionMergeState::Left => self.left_iter.get(),
                    UnionMergeState::Right=> self.right_iter.get(),
                    UnionMergeState::None => None,  
        }
    }
}


// fn choose_from_iters_merge<'a,'b>(state : &MergeState, 
//         self.left_iter : &'a Box<dyn StreamingIterator<Item = Posting> + 'b>,
//         self.right_iter : &'a Box<dyn StreamingIterator<Item = Posting> + 'b>) -> Option<&'a Posting>{
//         match state {
//                     MergeState::Left  | 
//                     MergeState::BothLeftThenRight(false) | 
//                     MergeState::BothRightThenLeft(true) |
//                     MergeState::OneLeftThenSkipRight(false) => self.left_iter.get(),
        
//                     MergeState::Right | 
//                     MergeState::BothRightThenLeft(false) | 
//                     MergeState::BothLeftThenRight(true)  => self.right_iter.get(),
        
//                     MergeState::None => None,  
//                     _ => panic!() // should not receive any skips here          
//                 }
//     }

// pub trait LinearMergeIterator{

    // #[inline(always)]
    // fn get_logic<'a,'b>(state : &MergeState, 
    //     self.left_iter : &'a Box<dyn StreamingIterator<Item = Posting> + 'b>,
    //     self.right_iter : &'a Box<dyn StreamingIterator<Item = Posting> + 'b>) -> Option<&'a Posting>{
        // match state {
        //     MergeState::Left  | 
        //     MergeState::BothLeftThenRight(false) | 
        //     MergeState::BothRightThenLeft(true) |
        //     MergeState::OneLeftThenSkipRight(false) => self.left_iter.get(),

        //     MergeState::Right | 
        //     MergeState::BothRightThenLeft(false) | 
        //     MergeState::BothLeftThenRight(true)  => self.right_iter.get(),

        //     MergeState::None => None,  
        //     _ => panic!() // should not receive any skips here          
        // }
    // }



    // fn next_state(items : (Option<&Posting>, Option<&Posting>), state : &mut MergeState) -> bool;

    // #[inline(always)]
    // fn advance_state<'a,'b>(
    //     self.left_iter : &'a mut Box<dyn StreamingIterator<Item = Posting> + 'b>,
    //     self.right_iter : &'a mut Box<dyn StreamingIterator<Item = Posting> + 'b>,
    //     state : &'a mut MergeState) -> () 
    //     {
        // let items = match state {
        //     MergeState::Left | MergeState::SkipLeft | MergeState::BothRightThenLeft(true) => { // last time left side was 'get', advance it
        //         self.state = MergeState::Left;
        //         (self.left_iter.next(), self.right_iter.get())
        //     },
        //     MergeState::Right | MergeState::SkipRight | MergeState::OneLeftThenSkipRight(true) | MergeState::BothLeftThenRight(true) => { // last time right side was 'get', advance it 
        //         self.state = MergeState::Right;    
        //         (self.left_iter.get(), self.right_iter.next()) 
        //     },
        //     MergeState::None => {
        //         (self.left_iter.next(),self.right_iter.next())
        //     },
        //     MergeState::BothLeftThenRight(false) => { // completed left, need right
        //         self.state = MergeState::BothLeftThenRight(true);
        //         self.left_iter.next();
        //         return;
        //     }, 
        //     MergeState::BothRightThenLeft(false) => { // completed right, need left
        //         self.state = MergeState::BothRightThenLeft(true);
        //         self.right_iter.next();
        //         return;
        //     },
        //     MergeState::OneLeftThenSkipRight(false) => {
        //         self.state = MergeState::OneLeftThenSkipRight(true);
        //         (self.left_iter.next(),self.right_iter.next())
        //     },      
        // };

        // let mut quit = false;
        // while !quit {
        //     quit = Self::next_state(items,state);

        //     if let MergeState::SkipLeft | MergeState::SkipRight = state{
        //         return Self::advance_state(self.left_iter,self.right_iter, state)
        //     } 
        // }

    // }


    
// }

///fn distance_merge(a: Vec<Posting>, b: Vec<Posting>, dst: u32) -> Vec<Posting> {
// let mut iter_left = a.iter();
// let mut iter_right = b.iter();
// let mut curr_items = (iter_left.next(), iter_right.next());
// let mut out = Vec::new();

// loop {
//     let (l, r) = match curr_items {
//         (Some(_), None) => return out,
//         (None, Some(_)) => return out,
//         (Some(l), Some(r)) => (l, r),
//         (None, None) => break,
//     };

//     if l.document_id == r.document_id {
//         if r.position.overflowing_sub(l.position).0 <= dst {
//             out.push(*l); // only added at beginning
//             out.push(*r);

//             // consume all matches under distance, but not the first non match
//             out.extend(iter_right.peeking_take_while(|c| {
//                 c.document_id == l.document_id
//                     && c.position.overflowing_sub(l.position).0 <= dst
//             }));

//             // move to next possible match or away to next doc
//             curr_items.1 = iter_right.next();
//             // curr_items.0 = iter_left.next();
//         } else if l.position < r.position {
//             curr_items.0 = iter_left.next();
//         } else {
//             curr_items.1 = iter_right.next();
//         }
//     } else {
//         if l.document_id < r.document_id {
//             curr_items.0 = iter_left.next();
//         } else {
//             curr_items.1 = iter_right.next();
//         }
//     }
// }

// return out;
// }


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

pub struct IntersectionMergeStreamingIterator<'a> {
    pub left_iter:  Box<dyn StreamingIterator<Item = Posting> + 'a>,
    pub right_iter: Box<dyn StreamingIterator<Item = Posting> + 'a>,
    pub state: IntersectionMergeState,

} 

impl <'a> IntersectionMergeStreamingIterator<'a> {
   pub fn new(
       l:Box<dyn StreamingIterator<Item = Posting> + 'a>,
       r:Box<dyn StreamingIterator<Item = Posting> + 'a>,
   ) -> Self {
       Self {
        left_iter: l,
        right_iter: r,
        state: IntersectionMergeState::None,
    }
   }
} 

impl <'a>StreamingIterator for IntersectionMergeStreamingIterator<'a>{
    type Item = Posting;

    fn advance(&mut self) {
        let mut items = match self.state {
             IntersectionMergeState::RightThenLeftFinish => { // last time left side was 'get', advance it
                (self.left_iter.next(), self.right_iter.get())
            },
            IntersectionMergeState::LeftThenRightFinish => { // last time right side was 'get', advance it 
                (self.left_iter.get(), self.right_iter.next()) 
            },
            IntersectionMergeState::None => {
                (self.left_iter.next(),self.right_iter.next())
            },
            IntersectionMergeState::LeftThenRightStart => { // completed left, need right
                self.state = IntersectionMergeState::LeftThenRightFinish;
                self.left_iter.next();
                return;
            }, 
            IntersectionMergeState::RightThenLeftStart => { // completed right, need left
                self.state = IntersectionMergeState::RightThenLeftFinish;
                self.right_iter.next();
                return;
            }  
        };

        let mut skip_side;
        loop {
            match items {
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

            items = match skip_side {
                SkipSide::Left =>  (self.left_iter.next(),self.right_iter.get()),
                SkipSide::Right => (self.left_iter.get(), self.right_iter.next()),
            };
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        match self.state {
            IntersectionMergeState::LeftThenRightStart | IntersectionMergeState::RightThenLeftFinish => self.left_iter.get(),
            IntersectionMergeState::LeftThenRightFinish | IntersectionMergeState::RightThenLeftStart => self.right_iter.get(), 
            IntersectionMergeState::None => None,
        }
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

pub struct DifferenceMergeStreamingIterator<'a> {
    left_iter:  Box<dyn StreamingIterator<Item = Posting> + 'a>,
    right_iter: Box<dyn StreamingIterator<Item = Posting> + 'a>,
    state: DifferenceMergeState,
    highest_doc_r: i32,
} 

impl <'a> DifferenceMergeStreamingIterator<'a> {
    pub fn new(
        l:Box<dyn StreamingIterator<Item = Posting> + 'a>,
        r:Box<dyn StreamingIterator<Item = Posting> + 'a>,
    ) -> Self {
        Self {
         left_iter: l,
         right_iter: r,
         state: DifferenceMergeState::None,
         highest_doc_r: -1
     }
    }
} 

impl <'a>StreamingIterator for DifferenceMergeStreamingIterator<'a>{
    type Item = Posting;

    fn advance(&mut self) {
        let mut items = match self.state {
            DifferenceMergeState::Left => { // last time left side was 'get', advance it
                (self.left_iter.next(), self.right_iter.get())
            },
            DifferenceMergeState::LeftThenSkipRightFinish => { // last time right side was 'get', advance it 
                (self.left_iter.get(), self.right_iter.next()) 
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
            match items{
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

            items = match skip_side {
                SkipSide::Left =>  (self.left_iter.next(),self.right_iter.get()),
                SkipSide::Right => (self.left_iter.get(), self.right_iter.next()),
            };
        }
    }

    fn get(&self) -> Option<&Self::Item> {
        match self.state {
            DifferenceMergeState::Left | DifferenceMergeState::LeftThenSkipRightStart => self.left_iter.get(),
            DifferenceMergeState::None => None,  
            _ => panic!() // should not receive any skips here          
        }
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

pub struct DistanceMergeStreamingIterator<'a> {
    left_iter:  Box<dyn StreamingIterator<Item = Posting> + 'a>,
    right_iter: Box<dyn StreamingIterator<Item = Posting> + 'a>,
    state: DistanceMergeState,
    dst: u32,
    in_streak: bool,
    streak_start_pos : u32,
} 

impl <'a> DistanceMergeStreamingIterator<'a> {
    pub fn new(
        dst: u32,
        l:Box<dyn StreamingIterator<Item = Posting> + 'a>,
        r:Box<dyn StreamingIterator<Item = Posting> + 'a>,
    ) -> Self {
        Self {
         left_iter: l,
         right_iter: r,
         state: DistanceMergeState::None,
         dst: dst,
         in_streak: false,
         streak_start_pos: 0,
     }
    }
} 

impl <'a>StreamingIterator for DistanceMergeStreamingIterator<'a>{
    type Item = Posting;

    fn advance(&mut self) {
        let mut items = match self.state {
            DistanceMergeState::Right | DistanceMergeState::LeftThenRightFinish  => { // last time right side was 'get', advance it 
                (self.left_iter.get(), self.right_iter.next()) 
            },
            DistanceMergeState::None => {
                (self.left_iter.next(),self.right_iter.next())
            },
            DistanceMergeState::LeftThenRightStart => { // completed left, need right
                self.state = DistanceMergeState::LeftThenRightFinish;
                self.left_iter.next();
                return;
            },   
        };

        let mut skip_side;
        loop {
            match items{
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

            items = match skip_side {
                SkipSide::Left =>  (self.left_iter.next(),self.right_iter.get()),
                SkipSide::Right => (self.left_iter.get(), self.right_iter.next()),
            };
        }
    }
    

    fn get(&self) -> Option<&Self::Item> {
        match self.state {
            DistanceMergeState::LeftThenRightStart => self.left_iter.get(),
            DistanceMergeState::Right | 
            DistanceMergeState::LeftThenRightFinish  => self.right_iter.get(),
            DistanceMergeState::None => None,  
        }
    }
}