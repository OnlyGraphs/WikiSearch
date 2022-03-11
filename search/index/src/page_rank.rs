

// pub fn init_page_rank(ids: HashMap<u32, Vec<u32>>, init_value: f64) -> HashMap<u32, f64> {
    
//     // Initialize all page rank value to the same initial value (usually 1.0)
//     // ids --> a vector containing the ids of all documents in the index

//     // Initialize a HashMap to store the page ranks in
//     let mut page_rank_per_id : HashMap<u32, f64> = HashMap::new();

//     // For every key in ids, insert an entry into page_rank_per_id
//     for k in ids.keys() {
//         page_rank_per_id.insert(k, init_value);
//     }

//     return page_rank_per_id;
// } 

// pub fn update_page_rank(page: u32, d: f64, in_links: Vec<u32>, page_ranks: HashMap<u32, f64>, out_links: HashMap<u32, Vec<u32>>) -> f64 {

//     // Compute the page rank value of a single page
//     // page --> id of page for which to compute the page rank
//     // d --> damping factor

//     let page_rank = (1-d);
//     let mut summed = 0;
//     for page in in_links {
//         pr = page_ranks.get(page);
//         ca = out_links.get(page).len();
//         summed = summed + (pr/ca);
//     }

//     page_rank = page_rank*summed;

//     return page_rank;
// }