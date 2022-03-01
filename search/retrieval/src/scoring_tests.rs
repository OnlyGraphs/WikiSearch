use crate::scoring::{idf, tfidf_query, tfidf_term};
use index::PreIndex;
use index::utils::get_document_with_text;
use index::{
    index::{BasicIndex, Index},
};
use parser::ast::{BinaryOp, Query, UnaryOp};

#[test]
fn test_idf() {
    let df = 2347.0;
    let num_documents = 1000000;
    let expected = ((num_documents as f64) / df).log10();
    let actual = idf(df, num_documents);
    assert_eq!(expected, actual);
}

#[test]
fn test_tfidf_term() {
    let mut pre_idx = PreIndex::default();

    pre_idx.add_document(get_document_with_text(
        1,
        "Strictly non-aquatic mammals",
        vec![("", "aaa bbb")],
        "While this text mentions a whale it is in no way dedicated to aquatic mammals",
        vec!["eee fff"],
        "ggg hhh",
    ))
    .unwrap();

    pre_idx.add_document(get_document_with_text(
        2,
        "All about big whales",
        vec![("", "aaa bbb")],
        "There once was a big whale who ate a big tuna sandwich His name was Randal Steep He lived in the deep ocean",
        vec!["eee fff"],
        "ggg hhh",
    )).unwrap();

    let mut idx: Box<dyn Index> = BasicIndex::from_pre_index(pre_idx);

    let df_whale = 2.0;
    let tf_whale_1 = 1.0;
    let tf_whale_2 = 1.0;

    let num_documents = 2;

    let idf_whale = ((num_documents as f64) / df_whale).log10();

    let tfidf_whale_1 = (1.0 + (tf_whale_1 as f64).log10()) * idf_whale;
    let tfidf_whale_2 = (1.0 + (tf_whale_2 as f64).log10()) * idf_whale;

    assert_eq!(tfidf_whale_1, tfidf_term("whale", 1, &idx));
    assert_eq!(tfidf_whale_2, tfidf_term("whale", 2, &idx));
}

#[test]
fn test_tfidf_term_2() {
    let mut pre_idx = PreIndex::default();

    pre_idx.add_document(get_document_with_text(
        1,
        "Strictly non-aquatic mammals",
        vec![("", "aaa bbb")],
        "While this text mentions a whale it is in no way dedicated to aquatic mammals",
        vec!["eee fff"],
        "ggg hhh",
    ))
    .unwrap();

    pre_idx.add_document(get_document_with_text(
        2,
        "All about big whales",
        vec![("", "aaa bbb")],
        "There once was a big whale who ate a big tuna sandwich His name was Randal Steep He lived in the deep ocean",
        vec!["eee fff"],
        "ggg hhh",
    )).unwrap();

    let mut idx: Box<dyn Index> = BasicIndex::from_pre_index(pre_idx);

    let df_big = 1.0;
    let tf_big_2 = 2.0;

    let num_documents = 2;

    let idf_big = ((num_documents as f64) / df_big).log10();

    let tfidf_big_1 = 0.0;
    let tfidf_big_2 = (1.0 + (tf_big_2 as f64).log10()) * idf_big;

    assert_eq!(tfidf_big_1, tfidf_term("big", 1, &idx));
    assert_eq!(tfidf_big_2, tfidf_term("big", 2, &idx));
}

#[test]
fn test_tfidf_simple_phrase_query() {
    let mut pre_idx = PreIndex::default();
    pre_idx.add_document(get_document_with_text(
        1,
        "Strictly non aquatic mammals",
        vec![("", "aaa bbb")],
        "While this text mentions a whale it is in no way dedicated to aquatic mammals",
        vec!["eee fff"],
        "ggg hhh",
    ))
    .unwrap();

    pre_idx.add_document(get_document_with_text(
        2,
        "All about big whales",
        vec![("", "aaa bbb")],
        "There once was a big whale who ate a big tuna sandwich His name was Randal Steep He lived in the deep ocean",
        vec!["eee fff"],
        "ggg hhh",
    )).unwrap();

    let mut idx: Box<dyn Index> = BasicIndex::from_pre_index(pre_idx);

    let query = Box::new(Query::FreetextQuery {
        tokens: vec!["big".to_string(), "whale".to_string()],
    });

    let df_whale = 2.0;
    let df_big = 1.0;
    let tf_big_2 = 2;
    let tf_whale_1 = 1.0;
    let tf_whale_2 = 1.0;

    let num_documents = 2;

    let idf_whale = ((num_documents as f64) / df_whale).log10();
    let idf_big = ((num_documents as f64) / df_big).log10();

    let tfidf_1_expected = (1.0 + (tf_whale_1 as f64).log10()) * idf_whale + 0.0;
    let tfidf_2_expected = (1.0 + (tf_whale_2 as f64).log10()) * idf_whale
        + (1.0 + (tf_big_2 as f64).log10()) * idf_big;

    let tfidf_1_actual = tfidf_query(1, &query, &idx);
    let tfidf_2_actual = tfidf_query(2, &query, &idx);

    assert_eq!(tfidf_1_expected, tfidf_1_actual);
    assert_eq!(tfidf_2_expected, tfidf_2_actual);
}

#[test]
fn test_simple_binary_query() {
    let mut pre_idx = PreIndex::default();
    pre_idx.add_document(get_document_with_text(
        1,
        "Strictly non aquatic mammals",
        vec![("", "aaa bbb")],
        "While this text mentions a whale it is in no way dedicated to aquatic mammals",
        vec!["eee fff"],
        "ggg hhh",
    ))
    .unwrap();

    pre_idx.add_document(get_document_with_text(
        2,
        "All about big whales",
        vec![("", "aaa bbb")],
        "There once was a big whale who ate a big tuna sandwich His name was Randal Steep He lived in the deep ocean",
        vec!["eee fff"],
        "ggg hhh",
    )).unwrap();

    let mut idx: Box<dyn Index> = BasicIndex::from_pre_index(pre_idx);

    let query = Box::new(Query::BinaryQuery {
        op: BinaryOp::And,
        lhs: Box::new(Query::FreetextQuery {
            tokens: vec!["big".to_string()],
        }),
        rhs: Box::new(Query::FreetextQuery {
            tokens: vec!["whale".to_string()],
        }),
    });

    let df_whale = 2.0;
    let df_big = 1.0;
    let tf_big_2 = 2;
    let tf_whale_1 = 1.0;
    let tf_whale_2 = 1.0;

    let num_documents = 2;

    let idf_whale = ((num_documents as f64) / df_whale).log10();
    let idf_big = ((num_documents as f64) / df_big).log10();

    let tfidf_1_expected = (1.0 + (tf_whale_1 as f64).log10()) * idf_whale + 0.0;
    let tfidf_2_expected = (1.0 + (tf_whale_2 as f64).log10()) * idf_whale
        + (1.0 + (tf_big_2 as f64).log10()) * idf_big;

    let tfidf_1_actual = tfidf_query(1, &query, &idx);
    let tfidf_2_actual = tfidf_query(2, &query, &idx);

    assert_eq!(tfidf_1_expected, tfidf_1_actual);
    assert_eq!(tfidf_2_expected, tfidf_2_actual);
}

#[test]
fn test_nested_binary_query() {
    let mut pre_idx = PreIndex::default();
    pre_idx.add_document(get_document_with_text(
        1,
        "Strictly non aquatic mammals",
        vec![("", "aaa bbb")],
        "While this text mentions a whale it is in no way dedicated to aquatic mammals",
        vec!["eee fff"],
        "ggg hhh",
    ))
    .unwrap();

    pre_idx.add_document(get_document_with_text(
        2,
        "All about big whales",
        vec![("", "aaa bbb")],
        "There once was a big whale who ate a big tuna sandwich His name was Randal Steep He lived in the deep ocean",
        vec!["eee fff"],
        "ggg hhh",
    )).unwrap();

    let mut idx: Box<dyn Index> = BasicIndex::from_pre_index(pre_idx);

    let query = Box::new(Query::BinaryQuery {
        op: BinaryOp::And,
        lhs: Box::new(Query::FreetextQuery {
            tokens: vec!["big".to_string()],
        }),
        rhs: Box::new(Query::UnaryQuery {
            op: UnaryOp::Not,
            sub: Box::new(Query::FreetextQuery {
                tokens: vec!["whale".to_string()],
            }),
        }),
    });

    let df_whale = 2.0;
    let df_big = 1.0;
    let tf_big_2 = 2;
    let tf_whale_1 = 1.0;
    let tf_whale_2 = 1.0;

    let num_documents = 2;

    let idf_whale = ((num_documents as f64) / df_whale).log10();
    let idf_big = ((num_documents as f64) / df_big).log10();

    let tfidf_1_expected = -(1.0 + (tf_whale_1 as f64).log10()) * idf_whale + 0.0;
    let tfidf_2_expected = -(1.0 + (tf_whale_2 as f64).log10()) * idf_whale
        + (1.0 + (tf_big_2 as f64).log10()) * idf_big;

    let tfidf_1_actual = tfidf_query(1, &query, &idx);
    let tfidf_2_actual = tfidf_query(2, &query, &idx);

    assert_eq!(tfidf_1_expected, tfidf_1_actual);
    assert_eq!(tfidf_2_expected, tfidf_2_actual);
}

#[test]
fn test_simple_relation_query() {
    let mut pre_idx = PreIndex::default();
    pre_idx.add_document(get_document_with_text(
        1,
        "Strictly non aquatic mammals",
        vec![("", "aaa bbb")],
        "While this text mentions a whale it is in no way dedicated to aquatic mammals",
        vec!["eee fff"],
        "ggg hhh",
    ))
    .unwrap();

    let mut idx: Box<dyn Index> = BasicIndex::from_pre_index(pre_idx);

    let query = Box::new(Query::RelationQuery {
        root: 1,
        hops: 3,
        sub: None,
    });

    assert_eq!(0.0, tfidf_query(1, &query, &idx));
}

#[test]
fn test_nested_relation_query() {
    let mut pre_idx = PreIndex::default();


    pre_idx.add_document(get_document_with_text(
        1,
        "Strictly non aquatic mammals",
        vec![("", "aaa bbb")],
        "While this text mentions a whale it is in no way dedicated to aquatic mammals",
        vec!["eee fff"],
        "ggg hhh",
    ))
    .unwrap();

    pre_idx.add_document(get_document_with_text(
        2,
        "All about big whales",
        vec![("", "aaa bbb")],
        "There once was a big whale who ate a big tuna sandwich His name was Randal Steep He lived in the deep ocean",
        vec!["eee fff"],
        "ggg hhh",
    )).unwrap();

    let mut idx: Box<dyn Index> = BasicIndex::from_pre_index(pre_idx);

    let query = Box::new(Query::RelationQuery {
        root: 2,
        hops: 3,
        sub: Some(Box::new(Query::BinaryQuery {
            op: BinaryOp::And,
            lhs: Box::new(Query::FreetextQuery {
                tokens: vec!["big".to_string()],
            }),
            rhs: Box::new(Query::UnaryQuery {
                op: UnaryOp::Not,
                sub: Box::new(Query::FreetextQuery {
                    tokens: vec!["whale".to_string()],
                }),
            }),
        })),
    });

    let df_whale = 2.0;
    let df_big = 1.0;
    let tf_big_2 = 2;
    let tf_whale_1 = 1.0;
    let tf_whale_2 = 1.0;

    let num_documents = 2;

    let idf_whale = ((num_documents as f64) / df_whale).log10();
    let idf_big = ((num_documents as f64) / df_big).log10();

    let tfidf_1_expected = -(1.0 + (tf_whale_1 as f64).log10()) * idf_whale + 0.0;
    let tfidf_2_expected = -(1.0 + (tf_whale_2 as f64).log10()) * idf_whale
        + (1.0 + (tf_big_2 as f64).log10()) * idf_big;

    let tfidf_1_actual = tfidf_query(1, &query, &idx);
    let tfidf_2_actual = tfidf_query(2, &query, &idx);

    assert_eq!(tfidf_1_expected, tfidf_1_actual);
    assert_eq!(tfidf_2_expected, tfidf_2_actual);
}
