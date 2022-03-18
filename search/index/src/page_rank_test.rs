use crate::page_rank::{update_page_rank, update_all_page_ranks, compute_page_ranks};
use std::collections::HashMap;
use rand::Rng;

pub fn page_rank_converged(prev_pr: &HashMap<u32, f64>, current_pr: &HashMap<u32, f64>) -> bool {
    
    for (key, pr1) in prev_pr {
        let mut pr2 = pr1;
        let cpr2 = current_pr.get(&key);
        match cpr2 {
            Some(v) => pr2 = v,
            _ => pr2 = pr1,
        };

        if (pr2-pr1).abs() > 0.00000001 {
            return false;
        }
    }

    return true;
}


#[test]
fn test_update_page_rank_simple() {

    // Setup
    let linked_pages_2 = vec![1, 2];

    let mut current_pr = HashMap::new();
    current_pr.insert(1, 0.0);
    current_pr.insert(0, 0.0);
    current_pr.insert(2, 0.0);

    let mut old_pr = HashMap::new();
    old_pr.insert(1, 0.0);
    old_pr.insert(0, 0.0);
    old_pr.insert(2, 0.0);

    // Pages to which the pages that link to page 0 link
    let mut out_links = HashMap::new();
    out_links.insert(0, linked_pages_2);

    let page = 0;
    let d = 0.85;
    // Only page 1 links to page 0
    let in_links = vec![1];

    update_page_rank(page, d, &in_links, &mut current_pr, &out_links, &old_pr);
    let expected_pr = 0.15;
    let pr = current_pr.get(&page).unwrap();
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

    let mut old_pr = HashMap::new();
    old_pr.insert(0, 1.5);
    old_pr.insert(1, 0.3);
    old_pr.insert(2, 0.9);
    old_pr.insert(3, 1.0);

    update_page_rank(page, 0.85, &in_links, &mut current_pr, &out_links, &old_pr);
    let updated_pr = current_pr.get(&page).unwrap();

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

    let pr_converged = page_rank_converged(&prev_pr, &current_pr);

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
    current_pr.insert(0,1.2);
    current_pr.insert(1, 1.5);
    current_pr.insert(2, 5.9);
    current_pr.insert(3, 0.900000001);

    let pr_converged = page_rank_converged(&prev_pr, &current_pr);

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

    let pr_0 = 0.15+0.85*((current_pr.get(&1).unwrap()/2.0)+(current_pr.get(&2).unwrap()/2.0));
    let pr_1 = 0.15+0.85*((current_pr.get(&0).unwrap()/1.0)+(current_pr.get(&2).unwrap()/2.0));
    let pr_2 = 0.15+0.85*((current_pr.get(&1).unwrap()/2.0));
    let op_pr_0 = pr_0/(pr_0+pr_1+pr_2);
    let op_pr_1 = pr_1/(pr_0+pr_1+pr_2);
    let op_pr_2 = pr_2/(pr_0+pr_1+pr_2);

    update_all_page_ranks(&outgoing_links, &incoming_links, &mut current_pr, 0.85);

    let mut expected_pr = HashMap::new();
    expected_pr.insert(0, op_pr_0);
    expected_pr.insert(1, op_pr_1);
    expected_pr.insert(2, op_pr_2);

    assert!(page_rank_converged(&current_pr, &expected_pr));

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

    let pr_0 = 0.15+0.85*((current_pr.get(&1).unwrap()/2.0)+(current_pr.get(&2).unwrap()/2.0));
    let pr_1 = 0.15+0.85*((current_pr.get(&0).unwrap()/1.0)+(current_pr.get(&2).unwrap()/2.0));
    let pr_2 = 0.15+0.85*((current_pr.get(&1).unwrap()/2.0));
    let op_pr_0 = pr_0/(pr_0+pr_1+pr_2);
    let op_pr_1 = pr_1/(pr_0+pr_1+pr_2);
    let op_pr_2 = pr_2/(pr_0+pr_1+pr_2);

    update_all_page_ranks(&outgoing_links, &incoming_links, &mut current_pr, 0.85);

    let mut expected_pr = HashMap::new();
    expected_pr.insert(0, op_pr_0);
    expected_pr.insert(1, op_pr_1);
    expected_pr.insert(2, op_pr_2);

    assert!(page_rank_converged(&current_pr, &expected_pr));
}

#[test]
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
    current_pr.insert(1, 0.4099099099099099);
    current_pr.insert(0, 0.33333333333333337);
    current_pr.insert(2, 0.2567567567567568);

    let pr_0 = 0.15+0.85*((current_pr.get(&1).unwrap()/2.0)+(current_pr.get(&2).unwrap()/2.0));
    let pr_1 = 0.15+0.85*((current_pr.get(&0).unwrap()/1.0)+(current_pr.get(&2).unwrap()/2.0));
    let pr_2 = 0.15+0.85*((current_pr.get(&1).unwrap()/2.0));
    let op_pr_0 = pr_0/(pr_0+pr_1+pr_2);
    let op_pr_1 = pr_1/(pr_0+pr_1+pr_2);
    let op_pr_2 = pr_2/(pr_0+pr_1+pr_2);

    update_all_page_ranks(&outgoing_links, &incoming_links, &mut current_pr, 0.85);

    let mut expected_pr = HashMap::new();
    expected_pr.insert(0, op_pr_0);
    expected_pr.insert(1, op_pr_1);
    expected_pr.insert(2, op_pr_2);

    assert!(page_rank_converged(&current_pr, &expected_pr));
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

    let pr = compute_page_ranks(&outgoing_links, &incoming_links, 0.85);

}

#[test]
fn test_page_rank_converged_1() {

    let mut outgoing_links: HashMap<u32, Vec<u32>> = HashMap::new();
    outgoing_links.insert(0, vec![1]);
    outgoing_links.insert(1, vec![0, 2]);
    outgoing_links.insert(2, vec![0,1]);

    let mut incoming_links: HashMap<u32, Vec<u32>> = HashMap::new();
    incoming_links.insert(0, vec![1,2]);
    incoming_links.insert(1, vec![0,2]);
    incoming_links.insert(2, vec![1]);

    let d = 0.85;

    let mut page_rank = incoming_links.keys().map(|k| (*k,0.0)).collect::<HashMap<u32,f64>>();
    update_all_page_ranks(&outgoing_links, &incoming_links, &mut page_rank, d);

    let mut iterations = 1;
    loop {
        if update_all_page_ranks(&outgoing_links, &incoming_links,&mut page_rank, d){
            println!("Number of iterations until convergence: {}", iterations);
            break;
        } else {
            iterations += 1;
        }
    }

    for (key, value) in page_rank {
        println!("Page: {}, Page rank: {}", key, value);
    }
}

#[test]
fn test_page_rank_converged_2() {

    let mut outgoing_links: HashMap<u32, Vec<u32>> = HashMap::new();
    outgoing_links.insert(0, vec![1]);
    outgoing_links.insert(1, vec![0, 2]);
    outgoing_links.insert(2, vec![0,1,3]);
    outgoing_links.insert(3, vec![0,1,2]);

    let mut incoming_links: HashMap<u32, Vec<u32>> = HashMap::new();
    incoming_links.insert(0, vec![1,2,3]);
    incoming_links.insert(1, vec![0,2,3]);
    incoming_links.insert(2, vec![1,3]);
    incoming_links.insert(3, vec![2]);

    let d = 0.85;

    let mut page_rank = incoming_links.keys().map(|k| (*k,0.0)).collect::<HashMap<u32,f64>>();
    update_all_page_ranks(&outgoing_links, &incoming_links, &mut page_rank, d);

    let mut iterations = 1;
    loop {
        if update_all_page_ranks(&outgoing_links, &incoming_links,&mut page_rank, d){
            println!("Number of iterations until convergence: {}", iterations);
            break;
        } else {
            iterations += 1;
        }
    }

    for (key, value) in page_rank {
        println!("Page: {}, Page rank: {}", key, value);
    }
}

#[test]
fn test_page_rank_converged_3() {

    let mut outgoing_links: HashMap<u32, Vec<u32>> = HashMap::new();
    outgoing_links.insert(0, vec![1, 4]);
    outgoing_links.insert(1, vec![0, 2, 4]);
    outgoing_links.insert(2, vec![0,1,3, 4]);
    outgoing_links.insert(3, vec![0,1,2, 4]);
    outgoing_links.insert(4, vec![0]);

    let mut incoming_links: HashMap<u32, Vec<u32>> = HashMap::new();
    incoming_links.insert(0, vec![1,2,3, 4]);
    incoming_links.insert(1, vec![0,2,3]);
    incoming_links.insert(2, vec![1,3]);
    incoming_links.insert(3, vec![2]);
    incoming_links.insert(4, vec![0,1,2,3]);

    let d = 0.85;

    let mut page_rank = incoming_links.keys().map(|k| (*k,0.0)).collect::<HashMap<u32,f64>>();
    update_all_page_ranks(&outgoing_links, &incoming_links, &mut page_rank, d);

    let mut iterations = 1;
    loop {
        if update_all_page_ranks(&outgoing_links, &incoming_links,&mut page_rank, d){
            println!("Number of iterations until convergence: {}", iterations);
            break;
        } else {
            iterations += 1;
        }
    }

    for (key, value) in page_rank {
        println!("Page: {}, Page rank: {}", key, value);
    }
}


#[test]
fn test_page_rank_converged_4() {

    let number_of_pages = 100;

    let mut outgoing_links: HashMap<u32, Vec<u32>> = HashMap::new();
    let mut incoming_links: HashMap<u32, Vec<u32>> = HashMap::new();

    for i in 0..number_of_pages {
        incoming_links.insert(i, vec![]);
    }

    let mut rng = rand::thread_rng();

    for n in 0..number_of_pages {
        // Randomly decide on the number of out links
        let num_out_links: u32 = rng.gen_range(0,100);
        let mut out_links: Vec<u32> = Vec::with_capacity(num_out_links.try_into().unwrap());
        for i in 0..num_out_links {
            loop {
                let new_num = rng.gen_range(0,100);
                if !(out_links.contains(&new_num) || new_num == n) {
                    let current_in_links = incoming_links.get(&new_num).unwrap();
                    let mut updated_in_links = current_in_links.clone();
                    updated_in_links.append(&mut vec![i as u32]);
                    incoming_links.insert(new_num, updated_in_links);
                    out_links.push(new_num);
                    break;
                }
            }
        }
        outgoing_links.insert(n, out_links);
    }

    let d = 0.85;
    println!("Commencing page rank computation");
    let mut page_rank = incoming_links.keys().map(|k| (*k,0.0)).collect::<HashMap<u32,f64>>();
    update_all_page_ranks(&outgoing_links, &incoming_links, &mut page_rank, d);

    let mut iterations = 1;
    loop {
        if update_all_page_ranks(&outgoing_links, &incoming_links,&mut page_rank, d){
            println!("Number of iterations until convergence: {}", iterations);
            break;
        } else if iterations > 1000 {
            println!("Page rank did not converge!");
            break;
        } else {
            iterations += 1;
        }
    }
}