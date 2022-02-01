use either::{Either, Left};
use std::collections::HashMap;

#[derive(Debug)]
pub enum Domain {
    org,
}

pub enum IndexEncoding {
    none,
    delta_encoding,
    elias_gamma_code,
}

// stores an appearance of a token in an article
#[derive(Debug)]
pub struct Posting {
    document_id: u32, //TODO: double check memory requirements, highest article word count etc
    position: u32,
}

//TODO:
//Make sure you check for integer overflows. Or, implementing Delta encoding would mitigate any such problems.
#[derive(Debug)]
pub struct BasicIndex {
    // pub title: String,
    // pub domain: Domain,
    // lastUpdatedDate: NaiveDate,

    //TODO: store tokens in a map, and store references in all others
    pub postings: HashMap<String, Vec<Posting>>,
    pub doc_freq: HashMap<String, u32>,
    pub term_freq: HashMap<String, HashMap<u32, u32>>, // tf(doc,term) -> frequency in document
    pub links: Either<HashMap<u32, Vec<String>>, HashMap<String, Vec<u32>>>, // List of tuples, where each element is: (Doc id, (Word_pos start, word_pos end))
    pub categories: HashMap<u32, Vec<String>>,
    pub abstracts: HashMap<u32, String>,
    // pub citation_positions:  Vec<(u32, (u32,u32))>,
}

impl Default for BasicIndex {
    fn default() -> Self {
        BasicIndex {
            postings: HashMap::new(),
            doc_freq: HashMap::new(),
            categories: HashMap::new(),
            abstracts: HashMap::new(),
            links: Left(HashMap::new()),
            term_freq: HashMap::new(),
        }
    }
}

impl BasicIndex {
    fn add_posting(&mut self, token: String, docid: u32, word_pos: u32) {
        let docid_and_word_pos_tuple = (docid, word_pos);
        self.postings
            .entry(token.clone())
            .or_insert(Vec::<Posting>::new())
            .push(Posting {
                document_id: docid,
                position: word_pos,
            });

        let freq_map: &mut HashMap<u32, u32> = self
            .term_freq
            .entry(token.clone())
            .or_insert(HashMap::new());
        *freq_map.entry(docid).or_insert(0) += 1;
    }

    pub fn add_document(
        &mut self,
        text: &str,
        doc_id: u32,
        categories: &str,
        article_links: &str,
        article_abstract: &str,
    ) {
        let mut word_pos = 0;
        for token in text.split(" ") {
            self.add_posting(token.to_string(), doc_id, word_pos);
            *self.doc_freq.entry(token.to_string()).or_insert(0) += 1;
            word_pos += 1;
        }

        let mut link_titles: Vec<String> = Vec::new();
        for link in article_links.split(",") {
            link_titles.push(link.trim().to_string());
        }

        self.links
            .as_mut()
            .expect_left("Index is not in buildable state")
            .insert(doc_id, link_titles);
        self.abstracts.insert(doc_id, article_abstract.to_string());
    }
}
