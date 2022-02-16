use search_lib::collections::SmallPostingMap;
use search_lib::index::index::BasicIndex;
use search_lib::index::index::Index;
use search_lib::index_structs::Posting;
use search_lib::parser::ast::{BinaryOp, Query, UnaryOp};
use search_lib::search::search::execute_query;
use search_lib::utils::test_utils::get_document_with_text;
use search_lib::utils::utils::MemFootprintCalculator;
use std::fmt::Debug;
use std::fmt::Formatter;

#[test]
fn test_one_word_query() {
    let mut idx: Box<dyn Index> = Box::new(BasicIndex::<SmallPostingMap>::default());

    idx.add_document(get_document_with_text(
        3,
        "d3",
        vec![("", "aaa bbb")],
        "ccc ddd",
        vec!["eee ddd"],
        "ggg hhh",
    ))
    .unwrap();

    idx.add_document(get_document_with_text(
        2,
        "d2",
        vec![("", "aaa bbb")],
        "ccc ddd",
        vec!["eee ddd"],
        "ggg hhh",
    ))
    .unwrap();

    idx.finalize().unwrap();

    assert_eq!(
        execute_query(
            Box::new(Query::FreetextQuery {
                tokens: vec!["ddd".to_string()]
            }),
            &idx
        ),
        vec![
            Posting {
                document_id: 2,
                position: 3
            },
            Posting {
                document_id: 2,
                position: 5
            },
            Posting {
                document_id: 3,
                position: 3
            },
            Posting {
                document_id: 3,
                position: 5
            }
        ]
    );
}

#[test]
fn test_and_query() {
    let mut idx: Box<dyn Index> = Box::new(BasicIndex::<SmallPostingMap>::default());

    idx.add_document(get_document_with_text(
        3,
        "d3",
        vec![("", "aaa bbb")],
        "ccc hello",
        vec!["eee world"],
        "ggg hhh",
    ))
    .unwrap();

    idx.add_document(get_document_with_text(
        2,
        "d2",
        vec![("", "iii jjj")],
        "hello lll",
        vec!["mmm nnn"],
        "ooo ppp",
    ))
    .unwrap();

    idx.finalize().unwrap();

    assert_eq!(
        execute_query(
            Box::new(Query::BinaryQuery {
                op: BinaryOp::And,
                lhs: Box::new(Query::FreetextQuery {
                    tokens: vec!["hello".to_string()]
                }),
                rhs: Box::new(Query::FreetextQuery {
                    tokens: vec!["world".to_string()]
                }),
            }),
            &idx
        ),
        vec![
            Posting {
                document_id: 3,
                position: 3
            },
            Posting {
                document_id: 3,
                position: 5
            },
        ]
    );
}

#[test]
fn test_multiple_word_query_same_as_or() {
    let mut idx: Box<dyn Index> = Box::new(BasicIndex::<SmallPostingMap>::default());

    idx.add_document(get_document_with_text(
        3,
        "d3",
        vec![("", "aaa bbb")],
        "ccc hello",
        vec!["eee world"],
        "ggg hhh",
    ))
    .unwrap();

    idx.add_document(get_document_with_text(
        2,
        "d2",
        vec![("", "iii jjj")],
        "hello lll",
        vec!["mmm nnn"],
        "ooo ppp",
    ))
    .unwrap();

    idx.finalize().unwrap();

    assert_eq!(
        execute_query(
            Box::new(Query::FreetextQuery {
                tokens: vec!["hello".to_string(), "world".to_string()]
            }),
            &idx
        ),
        execute_query(
            Box::new(Query::BinaryQuery {
                op: BinaryOp::Or,
                lhs: Box::new(Query::FreetextQuery {
                    tokens: vec!["hello".to_string()]
                }),
                rhs: Box::new(Query::FreetextQuery {
                    tokens: vec!["world".to_string()]
                }),
            }),
            &idx
        )
    );
}


#[test]
fn test_not_query() {
    let mut idx: Box<dyn Index> = Box::new(BasicIndex::<SmallPostingMap>::default());

    idx.add_document(get_document_with_text(
        3,
        "d3",
        vec![("", "aaa bbb")],
        "ccc hello",
        vec!["eee world"],
        "ggg hhh",
    ))
    .unwrap();

    idx.add_document(get_document_with_text(
        2,
        "d2",
        vec![("", "iii jjj")],
        "hello lll",
        vec!["mmm nnn"],
        "ooo ppp",
    ))
    .unwrap();

    idx.finalize().unwrap();

    assert_eq!(
        execute_query(
            Box::new(Query::UnaryQuery {
                op: UnaryOp::Not,
                sub: Box::new(Query::FreetextQuery {
                    tokens: vec!["world".to_string()]
                })
            }),
            &idx
        ),
        vec![
            Posting {
                document_id: 2,
                position: 0
            },
            Posting {
                document_id: 2,
                position: 1
            },
            Posting {
                document_id: 2,
                position: 2
            },
            Posting {
                document_id: 2,
                position: 3
            },
            Posting {
                document_id: 2,
                position: 4
            },
            Posting {
                document_id: 2,
                position: 5
            },
            Posting {
                document_id: 2,
                position: 6
            },
            Posting {
                document_id: 2,
                position: 7
            },
        ]
    );
}


#[test]
fn test_distance_query() {
    let mut idx: Box<dyn Index> = Box::new(BasicIndex::<SmallPostingMap>::default());

    idx.add_document(get_document_with_text(
        3,
        "d3",
        vec![("", "world hello")],
        "hello world",
        vec!["world world"],
        "ggg hhh",
    ))
    .unwrap();

    idx.add_document(get_document_with_text(
        2,
        "d2",
        vec![("", "iii world")],
        "hello lll",
        vec!["hello world"],
        "ooo ppp",
    ))
    .unwrap();

    idx.finalize().unwrap();

    assert_eq!(
        execute_query(
            Box::new(Query::DistanceQuery {
                dst: 2,
                lhs: "hello".to_string(),
                rhs: "world".to_string(),
                }
            ),
            &idx
        ),
        vec![
            Posting {
                document_id: 3,
                position: 1
            },
            Posting {
                document_id: 3,
                position: 2
            },
            Posting {
                document_id: 3,
                position: 3
            },
            Posting {
                document_id: 3,
                position: 4
            },
        ]
    );
}


#[test]
fn test_phrase_query() {
    let mut idx: Box<dyn Index> = Box::new(BasicIndex::<SmallPostingMap>::default());

    idx.add_document(get_document_with_text(
        3,
        "d3",
        vec![("", "world hello")],
        "hello world",
        vec!["eee world"],
        "ggg hhh",
    ))
    .unwrap();

    idx.add_document(get_document_with_text(
        2,
        "d2",
        vec![("", "iii world")],
        "hello lll",
        vec!["hello world"],
        "ooo ppp",
    ))
    .unwrap();

    idx.finalize().unwrap();

    assert_eq!(
        execute_query(
            Box::new(Query::PhraseQuery {
                tks: vec!["hello".to_string(),"world".to_string()]
                }
            ),
            &idx
        ),
        vec![
            Posting {
                document_id: 2,
                position: 4
            },
            Posting {
                document_id: 2,
                position: 5
            },
            Posting {
                document_id: 3,
                position: 2
            },
            Posting {
                document_id: 3,
                position: 3
            },
        ]
    );
}

#[test]
fn test_phrase_query_multiple() {

    let mut idx: Box<dyn Index> = Box::new(BasicIndex::<SmallPostingMap>::default());

    idx.add_document(get_document_with_text(
        3,
        "d3",
        vec![("", "world hello")],
        "hello world momma",
        vec!["eee world"],
        "ggg hhh",
    ))
    .unwrap();

    idx.add_document(get_document_with_text(
        2,
        "d2",
        vec![("", "iii world")],
        "hello lll",
        vec!["hello world"],
        "ooo ppp",
    ))
    .unwrap();

    idx.finalize().unwrap();

    assert_eq!(
        execute_query(
            Box::new(Query::PhraseQuery {
                tks: vec!["hello".to_string(),"world".to_string(), "momma".to_string()]
                }
            ),
            &idx
        ),
        vec![
            Posting {
                document_id: 3,
                position: 2
            },
            Posting {
                document_id: 3,
                position: 3
            },
            Posting {
                document_id: 3,
                position: 4
            },
        ]
    );
}



#[test]
fn test_phrase_query_multiple_same_start() {

    let mut idx: Box<dyn Index> = Box::new(BasicIndex::<SmallPostingMap>::default());

    idx.add_document(get_document_with_text(
        3,
        "d3",
        vec![("", "hello world momma")],
        "fff eee ddd",
        vec!["eee world"],
        "hello world",
    ))
    .unwrap();

    idx.add_document(get_document_with_text(
        2,
        "d2",
        vec![("", "hello world momma")],
        "hello world",
        vec!["hello world"],
        "ooo ppp",
    ))
    .unwrap();

    idx.finalize().unwrap();

    assert_eq!(
        execute_query(
            Box::new(Query::PhraseQuery {
                tks: vec!["hello".to_string(),"world".to_string(), "momma".to_string()]
                }
            ),
            &idx
        ),
        vec![
            Posting {
                document_id: 2,
                position: 0
            },
            Posting {
                document_id: 2,
                position: 1
            },
            Posting {
                document_id: 2,
                position: 2
            },
            Posting {
                document_id: 3,
                position: 0
            },
            Posting {
                document_id: 3,
                position: 1
            },
            Posting {
                document_id: 3,
                position: 2
            },
        ]
    );
}