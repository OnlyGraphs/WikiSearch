use std::collections::HashMap;
use core::mem::size_of;
use crate::index::{
    index::{BasicIndex, Index},
    index_structs::{PosRange, Posting},
};
use crate::tests::test_utils::{get_document_with_links, get_document_with_text};
use crate::utils::utils::{MemFootprintCalculator};
use std::array::IntoIter;

// TODO: split tests by library
// add integration tests



#[test]
fn test_real_mem_primitives(){
    assert_eq!((0 as u32).real_mem(),4);
    assert_eq!((0 as u64).real_mem(),8);
    assert_eq!(("hello".to_string()).real_mem(),24 + 5);
    assert_eq!(("hello").real_mem(),16 + 5);
    assert_eq!((vec!["hello".to_string(),"hello".to_string()]).real_mem(),24 + 24 + 24 + 5 + 5);
    assert_eq!( HashMap::<_, _>::from_iter(IntoIter::new(
        [(1 as u32, 2 as u32), (3, 4)])).real_mem(),
        4*4 + 48);
}

#[test]
fn test_add_after_finalize(){
    let mut idx = BasicIndex::default();

    idx.add_document(get_document_with_text(
        2,
        "d0",
        vec![("", "aaa bbb")],
        "ccc ddd",
        vec!["eee fff"],
        "ggg hhh",
    )).unwrap();

    idx.finalize().unwrap();

    let res = idx.add_document(get_document_with_text(
        2,
        "d0",
        vec![("", "aaa bbb")],
        "ccc ddd",
        vec!["eee fff"],
        "ggg hhh",
    ));

    assert_eq!(res.is_err(),true);
}

#[test]
fn test_add_duplicate_article(){
    let mut idx = BasicIndex::default();

    idx.add_document(get_document_with_text(
        2,
        "d0",
        vec![("", "aaa bbb")],
        "ccc ddd",
        vec!["eee fff"],
        "ggg hhh",
    )).unwrap();

    let res = idx.add_document(get_document_with_text(
        2,
        "d0",
        vec![("", "aaa bbb")],
        "ccc ddd",
        vec!["eee fff"],
        "ggg hhh",
    ));

    assert_eq!(res.is_err(),true);
}

#[test]
fn test_basic_index_get_postings() {
    let mut idx = BasicIndex::default();

    idx.add_document(get_document_with_text(
        2,
        "d0",
        vec![("", "aaa bbb")],
        "ccc ddd",
        vec!["eee fff"],
        "ggg hhh",
    )).unwrap();

    assert_eq!(
        *idx.get_postings("aaa").unwrap(),
        vec![Posting {
            document_id: 2,
            position: 0,
        }]
    );

    assert_eq!(
        *idx.get_postings("ddd").unwrap(),
        vec![Posting {
            document_id: 2,
            position: 3,
        }]
    );

    assert_eq!(
        *idx.get_postings("ggg").unwrap(),
        vec![Posting {
            document_id: 2,
            position: 6,
        }]
    );

    assert_eq!(idx.get_postings("dick"), None);
}

#[test]
fn test_basic_index_get_extent() {
    let mut idx = BasicIndex::default();

    idx.add_document(get_document_with_text(
        2,
        "d0",
        vec![("infobox", "aaa bbb"), ("infobox2", "hello")],
        "ccc ddd",
        vec!["eee fff", "world", "eggs"],
        "ggg hhh",
    )).unwrap();

    assert_eq!(
        *idx.get_extent_for("infobox", &2).unwrap(),
        PosRange {
            start_pos: 0,
            end_pos: 2,
        }
    );

    assert_eq!(
        *idx.get_extent_for("infobox2", &2).unwrap(),
        PosRange {
            start_pos: 2,
            end_pos: 3,
        }
    );

    assert_eq!(
        *idx.get_extent_for("citation", &2).unwrap(),
        PosRange {
            start_pos: 5,
            end_pos: 9
        }
    );

    assert_eq!(
        *idx.get_extent_for("categories", &2).unwrap(),
        PosRange {
            start_pos: 9,
            end_pos: 11
        }
    );

    assert_eq!(idx.get_extent_for("asd", &2), None);
}

#[test]
fn test_basic_index_tf() {
    let mut idx = BasicIndex::default();

    idx.add_document(get_document_with_text(
        0,
        "d0",
        vec![("infobox", "hello world"), ("infobox2", "hello")],
        "eggs world",
        vec!["this that", "that", "eggs"],
        "hello world",
    )).unwrap();

    assert_eq!(idx.tf("hello", 0), 3);
    assert_eq!(idx.tf("world", 0), 3);
    assert_eq!(idx.tf("this", 0), 1);
    assert_eq!(idx.tf("that", 0), 2);
    assert_eq!(idx.tf("eggs", 0), 2);
    assert_eq!(idx.tf("kirby", 0), 0);
}

#[test]
fn test_basic_index_df() {
    let mut idx = BasicIndex::default();

    idx.add_document(get_document_with_text(
        0,
        "d0",
        vec![("infobox", "hello world"), ("infobox2", "hello")],
        "eggs world",
        vec!["this that", "that", "eggs"],
        "hello world",
    )).unwrap();

    idx.add_document(get_document_with_text(
        1,
        "d1",
        vec![("infobox", "aaa aa"), ("infobox2", "aaa")],
        "eggs world",
        vec!["aaa aa", "aaa", "aa"],
        "aaa aaa",
    )).unwrap();

    idx.finalize().unwrap();

    assert_eq!(idx.df("hello"), 1);
    assert_eq!(idx.df("world"), 2);
    assert_eq!(idx.df("this"), 1);
    assert_eq!(idx.df("that"), 1);
    assert_eq!(idx.df("eggs"), 2);
    assert_eq!(idx.df("kirby"), 0);
}

#[test]
fn test_basic_index_links() {
    let mut idx = BasicIndex::default();

    idx.add_document(get_document_with_links(0, "source", "target1,target2")).unwrap();
    idx.add_document(get_document_with_links(1, "target1", "a")).unwrap();
    idx.add_document(get_document_with_links(2, "target2", "source")).unwrap();

    idx.finalize().unwrap();

    assert_eq!(idx.get_links(0).unwrap(), vec![1, 2]);
    assert_eq!(idx.get_links(1).unwrap().len(), 0);
    assert_eq!(idx.get_links(2).unwrap(), vec![0]);

    assert_eq!(idx.id_to_title(0), Some(&"source".to_string()));
    assert_eq!(idx.title_to_id("source".to_string()), Some(0));

    assert_eq!(idx.id_to_title(1), Some(&"target1".to_string()));
    assert_eq!(idx.title_to_id("target1".to_string()), Some(1));

    assert_eq!(idx.id_to_title(2), Some(&"target2".to_string()));
    assert_eq!(idx.title_to_id("target2".to_string()), Some(2));
}
