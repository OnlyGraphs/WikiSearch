use index::{
    get_document_with_links, get_document_with_text, get_document_with_text_and_links, Index,
    Posting, PreIndex,
};
use parser::ast::{BinaryOp, Query, StructureElem, UnaryOp};
use retrieval::{execute_query, get_docs_within_hops, search::preprocess_query};
use std::collections::HashMap;

#[test]
fn test_single_word() {
    let mut q = Query::FreetextQuery {
        tokens: vec!["bOrIs".to_string()],
    };

    preprocess_query(&mut q).unwrap();

    assert_eq!(
        q,
        Query::FreetextQuery {
            tokens: vec!["bori".to_string()]
        }
    )
}

#[test]
fn test_multiple_words() {
    let mut q = Query::FreetextQuery {
        tokens: vec!["BarCeLOna snickers".to_string()],
    };

    preprocess_query(&mut q).unwrap();

    assert_eq!(
        q,
        Query::FreetextQuery {
            tokens: vec!["barcelona".to_string(), "snicker".to_string()]
        }
    )
}

#[test]
fn test_stop_word() {
    let mut q = Query::FreetextQuery {
        tokens: vec!["the".to_string()],
    };

    preprocess_query(&mut q).unwrap();

    assert_eq!(
        q,
        Query::FreetextQuery {
            tokens: Vec::default()
        }
    )
}

#[test]
fn test_binary_query() {
    let mut q = Query::BinaryQuery {
        lhs: Box::new(Query::FreetextQuery {
            tokens: vec!["the".to_string()],
        }),
        op: BinaryOp::And,
        rhs: Box::new(Query::FreetextQuery {
            tokens: vec!["bars".to_string()],
        }),
    };

    preprocess_query(&mut q).unwrap();

    assert_eq!(
        q,
        Query::BinaryQuery {
            lhs: Box::new(Query::FreetextQuery {
                tokens: Vec::default()
            }),
            op: BinaryOp::And,
            rhs: Box::new(Query::FreetextQuery {
                tokens: vec!["bar".to_string()]
            }),
        }
    )
}

#[test]
fn test_unary_query() {
    let mut q = Query::UnaryQuery {
        sub: Box::new(Query::FreetextQuery {
            tokens: vec!["the".to_string()],
        }),
        op: UnaryOp::Not,
    };

    preprocess_query(&mut q).unwrap();

    assert_eq!(
        q,
        Query::UnaryQuery {
            sub: Box::new(Query::FreetextQuery {
                tokens: Vec::default()
            }),
            op: UnaryOp::Not,
        }
    )
}

#[test]
fn test_struct_query() {
    let mut q = Query::StructureQuery {
        sub: Box::new(Query::FreetextQuery {
            tokens: vec!["the".to_string()],
        }),
        elem: StructureElem::Category,
    };

    preprocess_query(&mut q).unwrap();

    assert_eq!(
        q,
        Query::StructureQuery {
            sub: Box::new(Query::FreetextQuery {
                tokens: Vec::default()
            }),
            elem: StructureElem::Category
        }
    )
}

#[test]
fn test_relational_query() {
    let mut q = Query::RelationQuery {
        sub: Some(Box::new(Query::FreetextQuery {
            tokens: vec!["the".to_string()],
        })),
        root: 34,
        hops: 2,
    };

    preprocess_query(&mut q).unwrap();

    assert_eq!(
        q,
        Query::RelationQuery {
            sub: Some(Box::new(Query::FreetextQuery {
                tokens: Vec::default()
            })),
            root: 34, // cannot be preprocessed
            hops: 2,
        }
    )
}

#[test]
fn test_distance_query() {
    let mut q = Query::DistanceQuery {
        lhs: "worm".to_string(),
        dst: 2,
        rhs: "bars".to_string(),
    };

    preprocess_query(&mut q).unwrap();

    assert_eq!(
        q,
        Query::DistanceQuery {
            lhs: "worm".to_string(),
            dst: 2,
            rhs: "bar".to_string(),
        }
    )
}

#[test]
#[should_panic]
fn test_distance_query_error() {
    let mut q = Query::DistanceQuery {
        lhs: "the".to_string(),
        dst: 2,
        rhs: "bars".to_string(),
    };

    preprocess_query(&mut q).unwrap();
}

#[test]
fn test_phrase_query() {
    let mut q = Query::PhraseQuery {
        tks: vec!["the".to_string(), "bikes".to_string()],
    };

    preprocess_query(&mut q).unwrap();

    assert_eq!(
        q,
        Query::PhraseQuery {
            tks: vec!["bike".to_string()],
        }
    )
}

#[test]
fn test_wildcard_query() {
    let mut q = Query::WildcardQuery {
        prefix: "the".to_string(),
        postfix: "bArs".to_string(),
    };

    preprocess_query(&mut q).unwrap();

    assert_eq!(
        q,
        Query::WildcardQuery {
            prefix: "the".to_string(),
            postfix: "bars".to_string(),
        }
    )
}

macro_rules! map {
    ( $( ($x:expr,$y:expr) ),* ) => {  // Match zero or more comma delimited items
        {
            let mut temp_set = HashMap::new();  // Create a mutable HashSet
            $(
                temp_set.insert($x,$y); // Insert each item matched into the HashSet
            )*
            temp_set // Return the populated HashSet
        }
    };
}

#[test]
fn test_docs_within_hops_line() {
    let mut pre_idx = PreIndex::default();

    pre_idx
        .add_document(get_document_with_links(0, "A", "B"))
        .unwrap();
    pre_idx
        .add_document(get_document_with_links(1, "B", "C"))
        .unwrap();
    pre_idx
        .add_document(get_document_with_links(2, "C", "D"))
        .unwrap();
    pre_idx
        .add_document(get_document_with_links(3, "D", "E"))
        .unwrap();
    pre_idx
        .add_document(get_document_with_links(4, "E", ""))
        .unwrap();

    let idx = Index::from_pre_index(pre_idx);

    let mut out = HashMap::default();
    get_docs_within_hops(0, 1, &mut out, &idx);
    assert_eq!(out, map![(0, 0), (1, 1)]);
    out.clear();

    get_docs_within_hops(0, 2, &mut out, &idx);
    assert_eq!(out, map![(0, 0), (1, 1), (2, 2)]);
    out.clear();

    get_docs_within_hops(0, 3, &mut out, &idx);
    assert_eq!(out, map![(0, 0), (1, 1), (2, 2), (3, 3)]);
    out.clear();

    get_docs_within_hops(0, 4, &mut out, &idx);
    assert_eq!(out, map![(0, 0), (1, 1), (2, 2), (3, 3), (4, 4)]);
    out.clear();

    get_docs_within_hops(0, 5, &mut out, &idx);
    assert_eq!(out, map![(0, 0), (1, 1), (2, 2), (3, 3), (4, 4)]);
    out.clear();
}

#[test]
fn test_docs_within_hops_inverse_line() {
    let mut pre_idx = PreIndex::default();

    pre_idx
        .add_document(get_document_with_links(0, "A", ""))
        .unwrap();
    pre_idx
        .add_document(get_document_with_links(1, "B", "A"))
        .unwrap();
    pre_idx
        .add_document(get_document_with_links(2, "C", "B"))
        .unwrap();
    pre_idx
        .add_document(get_document_with_links(3, "D", "C"))
        .unwrap();
    pre_idx
        .add_document(get_document_with_links(4, "E", "D"))
        .unwrap();

    let idx = Index::from_pre_index(pre_idx);

    let mut out = HashMap::default();
    get_docs_within_hops(0, 1, &mut out, &idx);
    assert_eq!(out, map![(0, 0), (1, 1)]);
    out.clear();

    get_docs_within_hops(0, 2, &mut out, &idx);
    assert_eq!(out, map![(0, 0), (1, 1), (2, 2)]);
    out.clear();

    get_docs_within_hops(0, 3, &mut out, &idx);
    assert_eq!(out, map![(0, 0), (1, 1), (2, 2), (3, 3)]);
    out.clear();

    get_docs_within_hops(0, 4, &mut out, &idx);
    assert_eq!(out, map![(0, 0), (1, 1), (2, 2), (3, 3), (4, 4)]);
    out.clear();

    get_docs_within_hops(0, 5, &mut out, &idx);
    assert_eq!(out, map![(0, 0), (1, 1), (2, 2), (3, 3), (4, 4)]);
    out.clear();
}

#[test]
fn test_docs_within_hops_complex() {
    let mut pre_idx = PreIndex::default();
    //               2   3
    //               C - D
    //               |
    //         0 A - B 1
    //               |
    //               E 4

    pre_idx
        .add_document(get_document_with_links(0, "A", ""))
        .unwrap();
    pre_idx
        .add_document(get_document_with_links(1, "B", "A\tE"))
        .unwrap();
    pre_idx
        .add_document(get_document_with_links(2, "C", "B\tD"))
        .unwrap();
    pre_idx
        .add_document(get_document_with_links(3, "D", ""))
        .unwrap();
    pre_idx
        .add_document(get_document_with_links(4, "E", ""))
        .unwrap();

    let idx = Index::from_pre_index(pre_idx);

    let mut out = HashMap::default();
    // get_docs_within_hops(1,0, &mut out, &idx);
    // assert_eq!(out,map![(1,0)]);
    // out.clear();

    get_docs_within_hops(1, 1, &mut out, &idx);
    assert_eq!(out, map![(1, 0), (0, 1), (2, 1), (4, 1)]);
    out.clear();

    get_docs_within_hops(1, 2, &mut out, &idx);
    assert_eq!(out, map![(1, 0), (0, 1), (2, 1), (4, 1), (3, 2)]);
    out.clear();

    get_docs_within_hops(3, 2, &mut out, &idx);
    assert_eq!(out, map![(1, 2), (2, 1), (3, 0)]);
    out.clear();
}

#[test]
fn test_one_word_query() {
    let mut pre_idx = PreIndex::default();

    pre_idx
        .add_document(get_document_with_text(
            3,
            "d3",
            vec![("", "aaa bbb")],
            "ccc ddd",
            vec!["eee ddd"],
            "ggg hhh",
        ))
        .unwrap();

    pre_idx
        .add_document(get_document_with_text(
            2,
            "d2",
            vec![("", "aaa bbb")],
            "ccc ddd",
            vec!["eee ddd"],
            "ggg hhh",
        ))
        .unwrap();

    let idx = Index::from_pre_index(pre_idx);

    assert_eq!(
        execute_query(
            &Box::new(Query::FreetextQuery {
                tokens: vec!["ddd".to_string()]
            }),
            &idx
        )
        .collect::<Vec<Posting>>(),
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
    let mut pre_idx = PreIndex::default();

    pre_idx
        .add_document(get_document_with_text(
            3,
            "d3",
            vec![("", "aaa bbb")],
            "ccc hello",
            vec!["eee world"],
            "ggg hhh",
        ))
        .unwrap();

    pre_idx
        .add_document(get_document_with_text(
            2,
            "d2",
            vec![("", "iii aaa")],
            "hello lll",
            vec!["mmm nnn"],
            "world ppp",
        ))
        .unwrap();

    let idx = Index::from_pre_index(pre_idx);

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
        )
        .collect::<Vec<Posting>>(),
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
    let mut pre_idx = PreIndex::default();

    pre_idx
        .add_document(get_document_with_text(
            3,
            "d3",
            vec![("", "aaa bbb")],
            "ccc hello",
            vec!["eee world"],
            "ggg hhh",
        ))
        .unwrap();

    pre_idx
        .add_document(get_document_with_text(
            2,
            "d2",
            vec![("", "iii jjj")],
            "hello lll",
            vec!["mmm nnn"],
            "ooo ppp",
        ))
        .unwrap();

    let idx = Index::from_pre_index(pre_idx);

    assert_eq!(
        execute_query(
            &Box::new(Query::FreetextQuery {
                tokens: vec!["hello".to_string(), "world".to_string()]
            }),
            &idx
        )
        .collect::<Vec<Posting>>(),
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
        .collect::<Vec<Posting>>()
    );
}

// #[test]
// fn test_not_query() {
//     let mut pre_idx= PreIndex::default();

//     pre_idx.add_document(get_document_with_text(
//         3,
//         "d3",
//         vec![("", "aaa bbb")],
//         "ccc hello",
//         vec!["eee world"],
//         "ggg hhh",
//     ))
//     .unwrap();

//     pre_idx.add_document(get_document_with_text(
//         2,
//         "d2",
//         vec![("", "iii jjj")],
//         "hello lll",
//         vec!["mmm nnn"],
//         "ooo ppp",
//     ))
//     .unwrap();

//     let idx = Index::from_pre_index(pre_idx);

//     assert_eq!(
//         execute_query(
//             &Box::new(Query::UnaryQuery {
//                 op: UnaryOp::Not,
//                 sub: Box::new(Query::FreetextQuery {
//                     tokens: vec!["world".to_string()]
//                 })
//             }),
//             &idx
//         ).collect::<Vec<Posting>>(),
//         vec![
//             Posting {
//                 document_id: 2,
//                 position: 0
//             },
//             Posting {
//                 document_id: 2,
//                 position: 1
//             },
//             Posting {
//                 document_id: 2,
//                 position: 2
//             },
//             Posting {
//                 document_id: 2,
//                 position: 3
//             },
//             Posting {
//                 document_id: 2,
//                 position: 4
//             },
//             Posting {
//                 document_id: 2,
//                 position: 5
//             },
//             Posting {
//                 document_id: 2,
//                 position: 6
//             },
//             Posting {
//                 document_id: 2,
//                 position: 7
//             },
//         ]
//     );
// }

#[test]
fn test_distance_query_execute() {
    let mut pre_idx = PreIndex::default();

    pre_idx
        .add_document(get_document_with_text(
            3,
            "d3",
            vec![("", "world hello")],
            "hello ddd",
            vec!["world world"],
            "ggg hhh",
        ))
        .unwrap();

    pre_idx
        .add_document(get_document_with_text(
            2,
            "d2",
            vec![("", "iii world")],
            "hello lll",
            vec!["hello world"],
            "ooo ppp",
        ))
        .unwrap();

    let idx = Index::from_pre_index(pre_idx);

    assert_eq!(
        execute_query(
            &Box::new(Query::DistanceQuery {
                dst: 2,
                lhs: "hello".to_string(),
                rhs: "world".to_string(),
            }),
            &idx
        )
        .collect::<Vec<Posting>>(),
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
            },
            Posting {
                document_id: 3,
                position: 5
            }
        ]
    );
}

#[test]
fn test_distance_query_overlap() {
    let mut pre_idx = PreIndex::default();

    pre_idx
        .add_document(get_document_with_text(
            3,
            "d3",
            vec![("", "dddd dddd")],
            "hello ddd",
            vec!["world world"],
            "ggg hhh",
        ))
        .unwrap();

    let idx = Index::from_pre_index(pre_idx);

    assert_eq!(
        execute_query(
            &Box::new(Query::DistanceQuery {
                dst: 3,
                lhs: "hello".to_string(),
                rhs: "world".to_string(),
            }),
            &idx
        )
        .collect::<Vec<Posting>>(),
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
fn test_phrase_query_execute() {
    let mut pre_idx = PreIndex::default();

    pre_idx
        .add_document(get_document_with_text(
            3,
            "d3",
            vec![("", "world hello")],
            "hello world",
            vec!["eee world"],
            "ggg hhh",
        ))
        .unwrap();

    pre_idx
        .add_document(get_document_with_text(
            2,
            "d2",
            vec![("", "iii world")],
            "hello lll",
            vec!["hello world"],
            "ooo ppp",
        ))
        .unwrap();

    let idx = Index::from_pre_index(pre_idx);

    assert_eq!(
        execute_query(
            &Box::new(Query::PhraseQuery {
                tks: vec!["hello".to_string(), "world".to_string()]
            }),
            &idx
        )
        .collect::<Vec<Posting>>(),
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
    let mut pre_idx = PreIndex::default();

    pre_idx
        .add_document(get_document_with_text(
            3,
            "d3",
            vec![("", "world hello")],
            "hello world momma",
            vec!["eee world"],
            "ggg hhh",
        ))
        .unwrap();

    pre_idx
        .add_document(get_document_with_text(
            2,
            "d2",
            vec![("", "iii world")],
            "hello lll",
            vec!["hello world"],
            "ooo ppp",
        ))
        .unwrap();

    let idx = Index::from_pre_index(pre_idx);

    let mut out = execute_query(
        &Box::new(Query::PhraseQuery {
            tks: vec![
                "hello".to_string(),
                "world".to_string(),
                "momma".to_string(),
            ],
        }),
        &idx,
    )
    .collect::<Vec<Posting>>();

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
    let mut pre_idx = PreIndex::default();

    pre_idx
        .add_document(get_document_with_text(
            3,
            "d3",
            vec![("", "hello world momma")],
            "fff eee ddd",
            vec!["eee world"],
            "hello world",
        ))
        .unwrap();

    pre_idx
        .add_document(get_document_with_text(
            2,
            "d2",
            vec![("", "hello world momma")],
            "hello world",
            vec!["hello world"],
            "ooo ppp",
        ))
        .unwrap();

    let idx = Index::from_pre_index(pre_idx);

    let mut out = execute_query(
        &Box::new(Query::PhraseQuery {
            tks: vec![
                "hello".to_string(),
                "world".to_string(),
                "momma".to_string(),
            ],
        }),
        &idx,
    )
    .collect::<Vec<Posting>>();
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
    let mut pre_idx = PreIndex::default();

    pre_idx
        .add_document(get_document_with_text(
            3,
            "d3",
            vec![("", "aaa bbb")],
            "hello world",
            vec!["hello world"],
            "ggg hhh",
        ))
        .unwrap();

    pre_idx
        .add_document(get_document_with_text(
            2,
            "d2",
            vec![("", "hello world")],
            "hello world",
            vec!["ddd ddd"],
            "ooo ppp",
        ))
        .unwrap();

    let idx = Index::from_pre_index(pre_idx);

    assert_eq!(
        execute_query(
            &Box::new(Query::StructureQuery {
                elem: StructureElem::Citation,
                sub: Box::new(Query::FreetextQuery {
                    tokens: vec!["hello".to_string(), "world".to_string()]
                })
            }),
            &idx
        )
        .collect::<Vec<Posting>>(),
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
    let mut pre_idx = PreIndex::default();

    pre_idx
        .add_document(get_document_with_text_and_links(
            0,
            "A",
            vec![("", "aaa hello")],
            "helasdlo world",
            vec!["asd world"],
            "ggg hhh",
            "B",
        ))
        .unwrap();

    pre_idx
        .add_document(get_document_with_text_and_links(
            1,
            "B",
            vec![("", "hello world")],
            "asd asd",
            vec!["ddd ddd"],
            "ooo ppp",
            "",
        ))
        .unwrap();

    pre_idx
        .add_document(get_document_with_text_and_links(
            2,
            "C",
            vec![("", "hello world")],
            "asd world",
            vec!["ddd ddd"],
            "ooo ppp",
            "B\tD",
        ))
        .unwrap();

    pre_idx
        .add_document(get_document_with_text_and_links(
            3,
            "D",
            vec![("", "hello world")],
            "asd world",
            vec!["ddd ddd"],
            "ooo ppp",
            "",
        ))
        .unwrap();

    let idx = Index::from_pre_index(pre_idx);

    let q = |i| {
        Box::new(Query::RelationQuery {
            root: 0,
            hops: i,
            sub: Some(Box::new(Query::FreetextQuery {
                tokens: vec!["hello".to_string()],
            })),
        })
    };

    assert_eq!(
        execute_query(&q(0), &idx).collect::<Vec<Posting>>(),
        vec![Posting {
            document_id: 0,
            position: 1
        }]
    );

    assert_eq!(
        execute_query(&q(1), &idx).collect::<Vec<Posting>>(),
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
        execute_query(&q(2), &idx).collect::<Vec<Posting>>(),
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
        execute_query(&q(3), &idx).collect::<Vec<Posting>>(),
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
        execute_query(
            &Box::new(Query::RelationQuery {
                root: 0,
                hops: 3,
                sub: None
            }),
            &idx
        )
        .collect::<Vec<Posting>>(),
        vec![
            Posting {
                document_id: 0,
                position: 0
            },
            Posting {
                document_id: 1,
                position: 1
            },
            Posting {
                document_id: 2,
                position: 2
            },
            Posting {
                document_id: 3,
                position: 3
            }
        ]
    );
}
