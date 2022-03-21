use std::{collections::HashMap, cmp::max, time::Instant};

use log::info;

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
/// returns the updated page rank
pub fn update_page_rank(page: u32, d: f64, in_links: &Vec<u32>, page_ranks: &mut HashMap<u32, f64>, out_links: &HashMap<u32, Vec<u32>>, prev_page_ranks: &HashMap<u32, f64>) -> f64 {

    //let previous_page_rank = *page_ranks.get(&page).unwrap(); // guaranteed to exist
    let mut page_rank = d/(page_ranks.keys().len() as f64);
    let mut summed = 0.0;
    let mut delta = 0.0;
    for page in in_links {
        let pr = prev_page_ranks.get(&page).unwrap_or(&0.0);
    
        // the number of outgoing links for any page in the list should be at least one, so set it to that by default
        // however some pages might not satisfy this, force this number to avoid div by zero
        let ca_len = max(
            out_links.get(&page)
                .map(|v| v.len()).unwrap_or(1)
            ,1) as f64;

        summed = summed + (pr/ca_len);
    }
    //println!("Interim: {}, {}, {}", page_rank, (1.0-d), summed);
    page_rank = page_rank + (1.0-d)*(summed as f64);
    page_ranks.insert(page,page_rank); // should always replace an old key

    return (page_rank-prev_page_ranks.get(&page).unwrap()).abs()
}

pub fn softmax(page_ranks: &mut HashMap<u32, f64>) {
    let mut pr_sum = 0.0;
    // Sum all the page ranks together
    page_ranks.iter_mut().for_each(|(page, page_rank)|  {
        // Add the current page rank to the total sum
        pr_sum += *page_rank;
    });

    page_ranks.iter_mut().for_each(|(page, page_rank)|  {
        // update the page rank by dividing the value by the total sum of page ranks
        let new_pr = *page_rank/pr_sum;
        *page_rank = new_pr;
    });

}

/// Performs an iteration of page rank
/// Returns true if converged false otherwise
pub fn update_all_page_ranks(outgoing_links: &HashMap<u32, Vec<u32>>, incoming_links: &HashMap<u32, Vec<u32>>, current_pr: &mut HashMap<u32, f64>, d: f64) -> bool {
    
    let old_pr = current_pr.clone();
    let mut delta = 0.0;

    for (page, in_links) in incoming_links {
        delta += update_page_rank(*page, d, in_links,current_pr, outgoing_links, &old_pr);
    }
    println!("Delta: {}", delta);
    return delta < 0.0001*(current_pr.keys().len() as f64);
}

pub fn compute_page_ranks(outgoing_links: &HashMap<u32, Vec<u32>>, incoming_links: &HashMap<u32, Vec<u32>>, d:f64) -> HashMap<u32, f64> {
    let mut page_rank = incoming_links.keys().map(|k| (*k,1.0/(incoming_links.keys().len() as f64))).collect::<HashMap<u32,f64>>();
    update_all_page_ranks(outgoing_links, incoming_links, &mut page_rank, d);

    let mut max_iters = std::env::var("PAGE_RANK_ITERS").unwrap_or("70".to_string())
        .parse::<u32>()
        .unwrap_or(70);

    let mut timer = Instant::now(); 
    loop {
        if max_iters <= 0 || update_all_page_ranks(outgoing_links, incoming_links,&mut page_rank, d){
            break;
        } else {
            max_iters -= 1;
        }
        info!("Page rank iterations left: {}, ({}s)",max_iters,timer.elapsed().as_secs());
        timer = Instant::now();
    }

    softmax(&mut page_rank);

    return page_rank;
}