use std::{collections::HashMap, cmp::max};

pub fn init_page_rank(ids: &HashMap<u32, Vec<u32>>, init_value: f64) -> HashMap<u32, f64> {
    // Initialize all page rank value to the same initial value (usually 1.0)
    // ids --> a vector containing the ids of all documents in the index
    // Initialize a HashMap to store the page ranks in
    let mut page_rank_per_id : HashMap<u32, f64> = HashMap::new();
    // For every key in ids, insert an entry into page_rank_per_id
    for k in ids.keys() {
        page_rank_per_id.insert(*k, init_value);
    }
    return page_rank_per_id;
}


/// Compute the page rank value of a single page
/// page --> id of page for which to compute the page rank
/// d --> damping factor
/// Returns the change in page rank from previous iteration
pub fn update_page_rank(page: u32, d: f64, in_links: &Vec<u32>,old_page_ranks: &HashMap<u32,f64>, page_ranks: &mut HashMap<u32, f64>, out_links: &HashMap<u32, Vec<u32>>) -> f64{

    let previous_page_rank = *page_ranks.get(&page).unwrap(); // guaranteed to exist
    let mut page_rank = 1.0-d;
    let mut summed = 0.0;
    for page in in_links {
        let pr = *old_page_ranks.get(&page).unwrap_or(&0.0);

        // the number of outgoing links for any page in the list should be at least one, so set it to that by default
        // however some pages might not satisfy this, force this number to avoid div by zero
        let ca_len = max(
            out_links.get(&page)
                .map(|v| v.len()).unwrap_or(1)
            ,1) as f64;

        println!("pr:{},ca_len:{}",pr,ca_len);

        summed = summed + (pr/ca_len);
    }
    println!("pr:{},d:{},summed:{}",page_rank,d,summed);
    page_rank = page_rank + d*(summed as f64);
    page_ranks.insert(page,page_rank); // should always replace an old key

    println!("page: {}, new_pr: {}",page,page_rank);

    return previous_page_rank - page_rank
}

/// Performs an iteration of page rank
/// Returns true if converged false otherwise
pub fn update_all_page_ranks(outgoing_links: &HashMap<u32, Vec<u32>>, incoming_links: &HashMap<u32, Vec<u32>>, current_pr: &mut HashMap<u32, f64>, d: f64) -> bool {
    
    let mut converged = true;
    let old_pr = current_pr.clone();

    for (page, in_links) in incoming_links {
        let delta = update_page_rank(*page, d, in_links,&old_pr, current_pr, outgoing_links);

        if delta.abs() > 0.000001 {
            converged = false;
        }    
    }

    return converged
}

pub fn compute_page_ranks(outgoing_links: &HashMap<u32, Vec<u32>>, incoming_links: &HashMap<u32, Vec<u32>>, d:f64) -> HashMap<u32, f64> {
    let mut page_rank = incoming_links.keys().map(|k| (*k,0.0)).collect::<HashMap<u32,f64>>();
    update_all_page_ranks(outgoing_links, incoming_links, &mut page_rank, d);

    loop {
        if update_all_page_ranks(outgoing_links, incoming_links,&mut page_rank, d){
            break;
        }
    }

    return page_rank;
}