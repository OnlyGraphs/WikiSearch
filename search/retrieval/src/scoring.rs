use index::index::Index;
use parser::ast::Query;

pub fn idf(df: f64, num_documents: u32) -> f64 {
    return ((num_documents as f64) / df).log10();
}

// Compute the tfidf for a single term
pub fn tfidf_term(term: &str, doc_id: u32, index: &Index) -> f64 {
    let num_documents = index.get_number_of_documents();
    let tf = index.tf(term, doc_id);
    let idf_term = idf(index.df(term) as f64, num_documents);
    if tf == 0 {
        return 0.0;
    } else {
        return (1.0 + (tf as f64).log10()) * idf_term;
    }
}

// compute the tfidf over a number of terms with regads to  certain document
pub fn tfidf_doc(terms: &Vec<String>, doc_id: u32, index: &Index) -> f64 {
    let mut score = 0.0;
    for term in terms {
        score += tfidf_term(term, doc_id, index);
    }
    return score;
}

pub fn tfidf_query(document_id: u32, query: &Box<Query>, index: &Index) -> f64 {
    match &**query {
        Query::FreetextQuery { tokens } => return tfidf_doc(&tokens, document_id, index),
        Query::BinaryQuery { op: _, lhs, rhs } => {
            return tfidf_query(document_id, &lhs, index) + tfidf_query(document_id, &rhs, index)
        }
        Query::UnaryQuery { op: _, sub } => return -1000*tfidf_query(document_id, &sub, index),
        Query::PhraseQuery { tks } => return tfidf_doc(&tks, document_id, index),
        Query::StructureQuery { elem: _, sub } => return tfidf_query(document_id, &sub, index),
        Query::RelationQuery {
            root: _,
            hops: _,
            sub,
        } => match sub {
            Some(v) => return tfidf_query(document_id, &v, index),
            _ => return 0.0,
        },
        _ => return 0.0,
    }
}

pub fn init_page_rank(ids: HashMap<u32, Vec<u32>>, init_value: f64) -> HashMap<u32, f64> {
    
    // Initialize all page rank value to the same initial value (usually 1.0)
    // ids --> a vector containing the ids of all documents in the index

    // Initialize a HashMap to store the page ranks in
    let mut page_rank_per_id : HashMap<u32, f64> = HashMap::new();

    // For every key in ids, insert an entry into page_rank_per_id
    for k in ids.keys() {
        page_rank_per_id.insert(k, init_value);
    }

    return page_rank_per_id;
} 

pub fn update_page_rank(page: u32, d: f64, in_links: Vec<u32>, page_ranks: HashMap<u32, f64>, out_links: HashMap<u32, Vec<u32>>) -> f64 {

    // Compute the page rank value of a single page
    // page --> id of page for which to compute the page rank
    // d --> damping factor

    let page_rank = (1-d);
    let mut summed = 0;
    for page in in_links {
        pr = page_ranks.get(page);
        ca = out_links.get(page).len();
        summed = summed + (pr/ca);
    }

    page_rank = page_rank*summed;

    return page_rank;
}