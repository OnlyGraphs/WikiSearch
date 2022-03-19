use crate::correct_query;
use crate::query_correction::investigate_query_naive_correction;
use index::get_document_with_text;
use index::{Index, PreIndex};
use parser::ast::Query;
use parser::{parse_binary_query, BinaryOp};
#[test]
fn test_retrieving_closest_token() {
    let mut pre_idx = PreIndex::default();

    pre_idx
        .add_document(get_document_with_text(
            0,
            "d0",
            vec![("infobox", "hello world"), ("infobox2", "hello")],
            "eggs world",
            vec!["this that", "that", "eggs"],
            "hello world",
        ))
        .unwrap();

    pre_idx
        .add_document(get_document_with_text(
            1,
            "d1",
            vec![("infobox", "aaa aa"), ("infobox2", "aaa")],
            "eggs world",
            vec!["aaa aa", "aaa", "aa"],
            "aaa aaa",
        ))
        .unwrap();

    let incorrect_token = "worldd";
    let idx = Index::from_pre_index(pre_idx);
    let tries = 1;
    let key_distance = 1;
    let key_distance_append_amount = 0;
    let postings_token_threshold = 0; // no threshold
    let based_on_postings_count = false;

    let new_token = investigate_query_naive_correction(
        &incorrect_token.to_string(),
        &idx,
        tries,
        key_distance,
        key_distance_append_amount,
        postings_token_threshold,
        based_on_postings_count,
    );
    assert_eq!(new_token, "world");
}

#[test]
fn test_correct_query() {
    let query = "worlddd AND hell";

    let (_s, binary_node) = parse_binary_query(query).unwrap();

    let mut pre_idx = PreIndex::default();

    pre_idx
        .add_document(get_document_with_text(
            0,
            "d0",
            vec![("infobox", "hello world"), ("infobox2", "hello")],
            "eggs world",
            vec!["this that", "that", "eggs"],
            "hello world",
        ))
        .unwrap();

    pre_idx
        .add_document(get_document_with_text(
            1,
            "d1",
            vec![("infobox", "aaa aa"), ("infobox2", "aaa")],
            "eggs world",
            vec!["aaa aa", "aaa", "aa"],
            "aaa aaa",
        ))
        .unwrap();

    let idx = Index::from_pre_index(pre_idx);
    //set field to false
    std::env::set_var("SUGGEST_MOST_APPEARANCES", "false");
    let target_q = Query::BinaryQuery {
        lhs: Box::new(Query::FreetextQuery {
            tokens: vec!["world".to_string()],
        }),
        op: BinaryOp::And,
        rhs: Box::new(Query::FreetextQuery {
            tokens: vec!["hello".to_string()],
        }),
    };
    let new_query = correct_query(&binary_node, &idx);
    assert_eq!(new_query, format!("{}", target_q));
}
