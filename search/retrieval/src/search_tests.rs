use std::collections::HashSet;
use index::collections::SmallPostingMap;
use index::index::BasicIndex;
use index::index::Index;
use index::index_structs::Posting;
use parser::ast::{BinaryOp, Query, StructureElem, UnaryOp};
use crate::search::execute_query;
use index::utils::{get_document_with_text, get_document_with_links, get_document_with_text_and_links};
use crate::search::{get_docs_within_hops};




macro_rules! set {
    ( $( $x:expr ),* ) => {  // Match zero or more comma delimited items
        {
            let mut temp_set = HashSet::new();  // Create a mutable HashSet
            $(
                temp_set.insert($x); // Insert each item matched into the HashSet
            )*
            temp_set // Return the populated HashSet
        }
    };
}

#[test]
fn test_docs_within_hops_line() {
    let mut idx : Box::<dyn Index> = Box::new(BasicIndex::<SmallPostingMap>::default());

    idx.add_document(get_document_with_links(0, "A", "B")) .unwrap();
    idx.add_document(get_document_with_links(1, "B", "C")).unwrap();
    idx.add_document(get_document_with_links(2, "C", "D")).unwrap();
    idx.add_document(get_document_with_links(3, "D", "E")).unwrap();
    idx.add_document(get_document_with_links(4, "E", "")).unwrap();

    idx.finalize().unwrap();

    let mut out = HashSet::default();
    get_docs_within_hops(0,1, &mut out, &idx);
    assert_eq!(out,set![0,1]);
    out.clear();

    get_docs_within_hops(0,2, &mut out, &idx);
    assert_eq!(out,set![0,1,2]);
    out.clear();

    get_docs_within_hops(0,3, &mut out, &idx);
    assert_eq!(out,set![0,1,2,3]);
    out.clear();

    get_docs_within_hops(0,4, &mut out, &idx);
    assert_eq!(out,set![0,1,2,3,4]);
    out.clear();

    get_docs_within_hops(0,5, &mut out, &idx);
    assert_eq!(out,set![0,1,2,3,4]);
    out.clear();
}

#[test]
fn test_docs_within_hops_inverse_line() {
    let mut idx : Box::<dyn Index> = Box::new(BasicIndex::<SmallPostingMap>::default());

    idx.add_document(get_document_with_links(0, "A", "")) .unwrap();
    idx.add_document(get_document_with_links(1, "B", "A")).unwrap();
    idx.add_document(get_document_with_links(2, "C", "B")).unwrap();
    idx.add_document(get_document_with_links(3, "D", "C")).unwrap();
    idx.add_document(get_document_with_links(4, "E", "D")).unwrap();

    idx.finalize().unwrap();

    let mut out = HashSet::default();
    get_docs_within_hops(0,1, &mut out, &idx);
    assert_eq!(out,set![0,1]);
    out.clear();

    get_docs_within_hops(0,2, &mut out, &idx);
    assert_eq!(out,set![0,1,2]);
    out.clear();

    get_docs_within_hops(0,3, &mut out, &idx);
    assert_eq!(out,set![0,1,2,3]);
    out.clear();

    get_docs_within_hops(0,4, &mut out, &idx);
    assert_eq!(out,set![0,1,2,3,4]);
    out.clear();

    get_docs_within_hops(0,5, &mut out, &idx);
    assert_eq!(out,set![0,1,2,3,4]);
    out.clear();
}

#[test]
fn test_docs_within_hops_complex() {
    let mut idx : Box::<dyn Index> = Box::new(BasicIndex::<SmallPostingMap>::default());

    //              C - D
    //              |
    //          A - B 
    //              |
    //              E

    idx.add_document(get_document_with_links(0, "A", "")) .unwrap();
    idx.add_document(get_document_with_links(1, "B", "A, E")).unwrap();
    idx.add_document(get_document_with_links(2, "C", "B, D")).unwrap();
    idx.add_document(get_document_with_links(3, "D", "")).unwrap();
    idx.add_document(get_document_with_links(4, "E", "")).unwrap();

    idx.finalize().unwrap();

    let mut out = HashSet::default();
    get_docs_within_hops(1,0, &mut out, &idx);
    assert_eq!(out,set![1]);
    out.clear();

    get_docs_within_hops(1,1, &mut out, &idx);
    assert_eq!(out,set![0,1,2,4]);
    out.clear();

    get_docs_within_hops(1,2, &mut out, &idx);
    assert_eq!(out,set![0,1,2,3,4]);
    out.clear();

    get_docs_within_hops(3,2, &mut out, &idx);
    assert_eq!(out,set![1,2,3]);
    out.clear();

}

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
            &Box::new(Query::FreetextQuery {
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
        vec![("", "iii aaa")],
        "hello lll",
        vec!["mmm nnn"],
        "world ppp",
    ))
    .unwrap();

    idx.finalize().unwrap();

    assert_eq!(
        execute_query(
            &Box::new(Query::BinaryQuery {
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
                document_id: 2,
                position: 2
            },
            Posting {
                document_id: 2,
                position: 6
            },
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
            &Box::new(Query::FreetextQuery {
                tokens: vec!["hello".to_string(), "world".to_string()]
            }),
            &idx
        ),
        execute_query(
            &Box::new(Query::BinaryQuery {
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
            &Box::new(Query::UnaryQuery {
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
        "hello ddd",
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
            &Box::new(Query::DistanceQuery {
                dst: 2,
                lhs: "hello".to_string(),
                rhs: "world".to_string(),
            }),
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
                position: 4
            }
        ]
    );
}

#[test]
fn test_distance_query_overlap() {
    let mut idx: Box<dyn Index> = Box::new(BasicIndex::<SmallPostingMap>::default());

    idx.add_document(get_document_with_text(
        3,
        "d3",
        vec![("", "dddd dddd")],
        "hello ddd",
        vec!["world world"],
        "ggg hhh",
    ))
    .unwrap();

    idx.finalize().unwrap();

    assert_eq!(
        execute_query(
            &Box::new(Query::DistanceQuery {
                dst: 3,
                lhs: "hello".to_string(),
                rhs: "world".to_string(),
            }),
            &idx
        ),
        vec![
            Posting {
                document_id: 3,
                position: 2
            },
            Posting {
                document_id: 3,
                position: 4
            },
            Posting {
                document_id: 3,
                position: 5
            }
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
            &Box::new(Query::PhraseQuery {
                tks: vec!["hello".to_string(), "world".to_string()]
            }),
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

    let mut out = execute_query(
        &Box::new(Query::PhraseQuery {
            tks: vec![
                "hello".to_string(),
                "world".to_string(),
                "momma".to_string(),
            ],
        }),
        &idx,
    );

    out.dedup(); // allow consecutive duplicates (due to overlaps)

    assert_eq!(
        out,
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

    let mut out = execute_query(
        &Box::new(Query::PhraseQuery {
            tks: vec![
                "hello".to_string(),
                "world".to_string(),
                "momma".to_string(),
            ],
        }),
        &idx,
    );
    out.dedup(); // allow consecutive duplicates due to overlaps

    assert_eq!(
        out,
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

#[test]
fn test_structure_search_citation() {
    let mut idx: Box<dyn Index> = Box::new(BasicIndex::<SmallPostingMap>::default());

    idx.add_document(get_document_with_text(
        3,
        "d3",
        vec![("", "aaa bbb")],
        "hello world",
        vec!["hello world"],
        "ggg hhh",
    ))
    .unwrap();

    idx.add_document(get_document_with_text(
        2,
        "d2",
        vec![("", "hello world")],
        "hello world",
        vec!["ddd ddd"],
        "ooo ppp",
    ))
    .unwrap();

    idx.finalize().unwrap();

    assert_eq!(
        execute_query(
            &Box::new(Query::StructureQuery {
                elem: StructureElem::Citation,
                sub: Box::new(Query::FreetextQuery {
                    tokens: vec!["hello".to_string(), "world".to_string()]
                })
            }),
            &idx
        ),
        vec![
            Posting {
                document_id: 3,
                position: 4
            },
            Posting {
                document_id: 3,
                position: 5
            },
        ]
    );
}


#[test]
fn test_relational_search() {
    let mut idx: Box<dyn Index> = Box::new(BasicIndex::<SmallPostingMap>::default());

    idx.add_document(get_document_with_text_and_links(
        0,
        "A",
        vec![("", "aaa hello")],
        "helasdlo world",
        vec!["asd world"],
        "ggg hhh",

        "B"
    ))
    .unwrap();

    idx.add_document(get_document_with_text_and_links(
        1,
        "B",
        vec![("", "hello world")],
        "asd asd",
        vec!["ddd ddd"],
        "ooo ppp",
        ""
    ))
    .unwrap();


    idx.add_document(get_document_with_text_and_links(
        2,
        "C",
        vec![("", "hello world")],
        "asd world",
        vec!["ddd ddd"],
        "ooo ppp",
        "B, D"
    ))
    .unwrap();

    idx.add_document(get_document_with_text_and_links(
        3,
        "D",
        vec![("", "hello world")],
        "asd world",
        vec!["ddd ddd"],
        "ooo ppp",
        ""
    ))
    .unwrap();

    idx.finalize().unwrap();

    let q = |i| Box::new(Query::RelationQuery {
        root: "A".to_string(),
        hops: i,
        sub: Some(Box::new(Query::FreetextQuery {
            tokens: vec!["hello".to_string()]
        }))
    });

    assert_eq!(
        execute_query(&q(0),&idx),
        vec![
            Posting {
                document_id: 0,
                position: 1
            }
        ]
    );

    assert_eq!(
        execute_query(&q(1),&idx),
        vec![
            Posting {
                document_id: 0,
                position: 1
            },
            Posting {
                document_id: 1,
                position: 0
            }
        ]
    );

    assert_eq!(
        execute_query(&q(2),&idx),
        vec![
            Posting {
                document_id: 0,
                position: 1
            },
            Posting {
                document_id: 1,
                position: 0
            },
            Posting {
                document_id: 2,
                position: 0
            }
        ]
    );


    assert_eq!(
        execute_query(&q(3),&idx),
        vec![
            Posting {
                document_id: 0,
                position: 1
            },
            Posting {
                document_id: 1,
                position: 0
            },
            Posting {
                document_id: 2,
                position: 0
            },
            Posting {
                document_id: 3,
                position: 0
            }
        ]
    );

    assert_eq!(
        execute_query(&Box::new(Query::RelationQuery {
                root: "A".to_string(),
                hops: 3,
                sub: None
            })
            ,&idx),
        vec![
            Posting {
                document_id: 0,
                position: 0
            },
            Posting {
                document_id: 1,
                position: 0
            },
            Posting {
                document_id: 2,
                position: 0
            },
            Posting {
                document_id: 3,
                position: 0
            }
        ]
    );
}