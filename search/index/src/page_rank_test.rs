use crate::page_rank::{init_page_rank, update_page_rank};
use std::collections::HashMap;

#[test]
fn test_init_page_rank() {
    let mut ids = HashMap::new();
    let linked_pages_1 = vec![0];
    let linked_pages_2 = vec![1, 2];
    let linked_pages_3 = vec![];
    ids.insert(1, linked_pages_1);
    ids.insert(0, linked_pages_2);
    ids.insert(2, linked_pages_3);
    let init_value = 0.0;

    let mut expected_pr = HashMap::new();
    expected_pr.insert(1, 0.0);
    expected_pr.insert(0, 0.0);
    expected_pr.insert(2, 0.0);

    let actual_pr = init_page_rank(ids, init_value);
    assert_eq!(expected_pr, actual_pr);

}

#[test]
fn test_update_page_rank_simple() {

    // Setup
    let linked_pages_2 = vec![1, 2];

    let mut current_pr = HashMap::new();
    current_pr.insert(1, 0.0);
    current_pr.insert(0, 0.0);
    current_pr.insert(2, 0.0);

    // Pages to which the pages that link to page 0 link
    let mut out_links = HashMap::new();
    out_links.insert(0, linked_pages_2);

    let page = 0;
    let d = 0.85;
    // Only page 1 links to page 0
    let in_links = vec![1];

    let pr = update_page_rank(page, d, in_links, current_pr, out_links);
    let expected_pr = 0.15;
    assert!((pr - expected_pr).abs() < 0.000001);
}

#[test]
fn test_update_page_rank_complex_1() {
    // Setup

    // Page for which we want to compute the page rank
    let page = 1;

    // Pages linking to page 1
    let in_links = vec![0, 2, 3];

    // Pages that the pages linking to page 1 link to
    let mut out_links = HashMap::new();
    out_links.insert(0, vec![1]);
    out_links.insert(2, vec![1, 0]);
    out_links.insert(3, vec![0, 1]);

    let mut current_pr = HashMap::new();
    current_pr.insert(0, 1.5);
    current_pr.insert(1, 0.3);
    current_pr.insert(2, 0.9);
    current_pr.insert(3, 1.0);

    let updated_pr = update_page_rank(page, 0.85, in_links, current_pr, out_links);

    let expected_pr = 2.2325;
    assert!((updated_pr-expected_pr).abs() < 0.000001);

}