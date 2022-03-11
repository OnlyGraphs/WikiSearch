use crate::page_rank::{init_page_rank, update_page_rank, page_rank_converged, update_all_page_ranks, compute_page_ranks};
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

#[test]
pub fn test_page_ranks_converged_true() {

    // Setup

    let mut prev_pr = HashMap::new();
    prev_pr.insert(0, 0.2775);
    prev_pr.insert(1, 0.21375);
    prev_pr.insert(2, 0.34125);

    let mut current_pr = HashMap::new();
    current_pr.insert(1, 0.21375000000000002);
    current_pr.insert(0,0.2775);
    current_pr.insert(2, 0.34125);

    let pr_converged = page_rank_converged(prev_pr, current_pr);

    assert!(pr_converged);
}

#[test]
pub fn test_page_ranks_converged_false() {

    // Setup

    let mut prev_pr = HashMap::new();
    prev_pr.insert(0, 1.0);
    prev_pr.insert(1, 1.5);
    prev_pr.insert(2, 5.9);
    prev_pr.insert(3, 0.9);

    let mut current_pr = HashMap::new();
    current_pr.insert(0,1.0001);
    current_pr.insert(1, 1.5);
    current_pr.insert(2, 5.9);
    current_pr.insert(3, 0.900000001);

    let pr_converged = page_rank_converged(prev_pr, current_pr);

    assert!(!pr_converged);
}

#[test]
pub fn test_update_page_rank_for_all_simple_step1() {

    let mut outgoing_links = HashMap::new();
    outgoing_links.insert(0, vec![1]);
    outgoing_links.insert(1, vec![0, 2]);
    outgoing_links.insert(2, vec![0,1]);

    let mut incoming_links = HashMap::new();
    incoming_links.insert(0, vec![1,2]);
    incoming_links.insert(1, vec![0, 2]);
    incoming_links.insert(2, vec![1]);

    let mut current_pr = HashMap::new();
    current_pr.insert(1, 0.0);
    current_pr.insert(0, 0.0);
    current_pr.insert(2, 0.0);

    let actual_pr = update_all_page_ranks(outgoing_links, incoming_links, current_pr, 0.85);

    let mut expected_pr = HashMap::new();
    expected_pr.insert(1, 0.15);
    expected_pr.insert(0, 0.15);
    expected_pr.insert(2, 0.15);

    assert!(page_rank_converged(actual_pr, expected_pr));
}

#[test]
pub fn test_update_page_rank_for_all_simple_step2() {

    let mut outgoing_links = HashMap::new();
    outgoing_links.insert(0, vec![1]);
    outgoing_links.insert(1, vec![0, 2]);
    outgoing_links.insert(2, vec![0,1]);

    let mut incoming_links = HashMap::new();
    incoming_links.insert(0, vec![1,2]);
    incoming_links.insert(1, vec![0, 2]);
    incoming_links.insert(2, vec![1]);

    let mut current_pr = HashMap::new();
    current_pr.insert(1, 0.15);
    current_pr.insert(0, 0.15);
    current_pr.insert(2, 0.15);

    let actual_pr = update_all_page_ranks(outgoing_links, incoming_links, current_pr, 0.85);
    let mut expected_pr = HashMap::new();
    expected_pr.insert(0, 0.2775);
    expected_pr.insert(1, 0.34125);
    expected_pr.insert(2, 0.21375);
    assert!(page_rank_converged(actual_pr, expected_pr));
}

pub fn test_update_page_rank_for_all_simple_step3() {

    let mut outgoing_links = HashMap::new();
    outgoing_links.insert(0, vec![1]);
    outgoing_links.insert(1, vec![0, 2]);
    outgoing_links.insert(2, vec![0,1]);

    let mut incoming_links = HashMap::new();
    incoming_links.insert(0, vec![1,2]);
    incoming_links.insert(1, vec![0, 2]);
    incoming_links.insert(2, vec![1]);

    let mut current_pr = HashMap::new();
    current_pr.insert(1, 0.2775);
    current_pr.insert(0, 0.34125);
    current_pr.insert(2, 0.21375);

    let actual_pr = update_all_page_ranks(outgoing_links, incoming_links, current_pr, 0.85);
    let mut expected_pr = HashMap::new();
    expected_pr.insert(0, 0.385875);
    expected_pr.insert(1, 0.47671875);
    expected_pr.insert(2, 0.29503125);
    assert!(page_rank_converged(actual_pr, expected_pr));
}

#[test]
fn test_calculate_page_rank() {
    let mut outgoing_links = HashMap::new();
    outgoing_links.insert(0, vec![1]);
    outgoing_links.insert(1, vec![0, 2]);
    outgoing_links.insert(2, vec![0,1]);

    let mut incoming_links = HashMap::new();
    incoming_links.insert(0, vec![1,2]);
    incoming_links.insert(1, vec![0, 2]);
    incoming_links.insert(2, vec![1]);

    let pr = compute_page_ranks(outgoing_links, incoming_links, 0.85);

    for (key, value) in pr {
        println!("Page {}: {}", key, value);
    }
}