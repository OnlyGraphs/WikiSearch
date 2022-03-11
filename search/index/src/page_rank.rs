use std::collections::HashMap;

pub fn init_page_rank(ids: HashMap<u32, Vec<u32>>, init_value: f64) -> HashMap<u32, f64> {
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

pub fn page_rank_converged(prev_pr: HashMap<u32, f64>, current_pr: HashMap<u32, f64>) -> bool {
    
    for (key, pr1) in prev_pr {
        let mut pr2 = pr1;
        let cpr2 = current_pr.get(&key);
        match cpr2 {
            Some(v) => pr2 = *v,
            _ => pr2 = pr1,
        };

        if (pr2-pr1).abs() > 0.000001 {
            return false;
        }
    }

    return true;
}

pub fn update_page_rank(page: u32, d: f64, in_links: Vec<u32>, page_ranks: HashMap<u32, f64>, out_links: HashMap<u32, Vec<u32>>) -> f64 {
    // Compute the page rank value of a single page
    // page --> id of page for which to compute the page rank
    // d --> damping factor
    let mut page_rank = 1.0-d;
    let mut summed = 0.0;
    for page in in_links {
        let pr_option = page_ranks.get(&page);
        let mut pr = 0.0;
        match pr_option {
            Some(v) => pr = *v,
            _ => pr = 0.0,
        }
        let ca = out_links.get(&page);

        // the number of outgoing links for any page in the list should be at least one, so set it to that by default
        let mut ca_len = 1.0;
        match ca {
            Some(v) => ca_len = v.len() as f64,
            _ => ca_len = 1.0,
        }
        summed = summed + (pr/ca_len);
    }
    page_rank = page_rank + d*(summed as f64);
    return page_rank;
}

pub fn update_all_page_ranks(outgoing_links: HashMap<u32, Vec<u32>>, incoming_links: HashMap<u32, Vec<u32>>, current_pr: HashMap<u32, f64>, d: f64) -> HashMap<u32, f64> {

    let mut page_ranks = HashMap::new();

    for (page, in_links) in incoming_links {
        let pr = update_page_rank(page, d, in_links, current_pr.clone(), outgoing_links.clone());
        page_ranks.insert(page, pr);
    }

    return page_ranks;
}

pub fn compute_page_ranks(outgoing_links: HashMap<u32, Vec<u32>>, incoming_links: HashMap<u32, Vec<u32>>, d:f64) -> HashMap<u32, f64> {
    let mut prev_pr = init_page_rank(incoming_links.clone(),0.0);
    let mut current_pr = update_all_page_ranks(outgoing_links.clone(), incoming_links.clone(), prev_pr.clone(), d);

    while !(page_rank_converged(prev_pr.clone(), current_pr.clone())) {
        prev_pr = current_pr.clone();
        current_pr = update_all_page_ranks(outgoing_links.clone(), incoming_links.clone(), prev_pr.clone(), d);
    }

    return current_pr;
}