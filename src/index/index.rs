use crate::index::index_structs::*;
use crate::index::utils::*;

use either::{Either, Left};
use std::collections::HashMap;

/**
 * BasicIndex Structure:
 * Metadata <NOT included in postings>
 * Abstracts <NOT included in postings>
 * Infobox <Word position starts here at 0>
 * Main body
 * Citations
 * Categories
 */

#[derive(Debug)]
pub enum Domain {
    Simple,
}

pub enum IndexEncoding {
    None,
    DeltaEncoding,
    EliasGammaCode,
}

//TODO: Interface to specify functions that should be shared among different types of indices created (Ternary Index Tree vs BasicIndex)
pub trait IndexInterface {
    fn add_tokens(&mut self, doc_id: u32, text_to_add: String, word_pos: u32) -> u32;

    fn add_posting(&mut self, token: String, docid: u32, word_pos: u32);
    fn add_document(&mut self, document: Document);

    fn set_dump_id(&mut self, new_dump_id: u32);
    fn add_document_metadata(
        &mut self,
        doc_id: u32,
        title: String,
        lastUpdatedDate: String,
        namespace: u32,
    );
    fn add_abstract(&mut self, doc_id: u32, article_abstract: String);

    fn add_links(&mut self, doc_id: u32, article_links: &str);

    //TODO: Change to &str
    fn add_infoboxes(
        &mut self,
        doc_id: u32,
        infobox_type: String,
        text: Vec<String>,
        infobox_ids: Vec<u32>,
        word_pos: u32,
    ) -> u32;
    fn add_main_text(&mut self, doc_id: u32, main_text: &str, word_pos: u32) -> u32;

    fn add_citations(
        &mut self,
        doc_id: u32,
        citations_body: Vec<String>,
        citation_ids: Vec<u32>,
        word_pos: u32,
    ) -> u32;
    fn add_categories(&mut self, doc_id: u32, categories_str: String, word_pos: u32) -> u32;
}

//TODO:
//Make sure you check for integer overflows. Or, implementing Delta encoding would mitigate any such problems.
#[derive(Debug)]
pub struct BasicIndex {
    pub dump_id: Option<u32>,
    pub document_metadata: HashMap<u32, DocumentMetaData>,
    //TODO: store tokens in a map, and store references in all others
    pub postings: HashMap<String, Vec<Posting>>,
    pub doc_freq: HashMap<String, u32>,
    pub term_freq: HashMap<String, HashMap<u32, u32>>, // tf(doc,term) -> frequency in document
    pub links: Either<HashMap<u32, Vec<String>>, HashMap<String, Vec<u32>>>, // List of tuples, where each element is: (Doc id, (Word_pos start, word_pos end))
    pub categories: HashMap<u32, Vec<ExtentCategories>>, //The name of category pages which a page links to  (eg. docid -> category1, category2).
    pub abstracts: HashMap<u32, String>,
    pub infoboxes: HashMap<u32, Vec<ExtentInfoBox>>,
    pub citations: HashMap<u32, Vec<ExtentCitations>>,
}

impl Default for BasicIndex {
    fn default() -> Self {
        BasicIndex {
            dump_id: None,
            postings: HashMap::new(),
            doc_freq: HashMap::new(),
            categories: HashMap::new(),
            abstracts: HashMap::new(),
            links: Left(HashMap::new()),
            term_freq: HashMap::new(),
            document_metadata: HashMap::new(),
            infoboxes: HashMap::new(),
            citations: HashMap::new(),
        }
    }
}

impl IndexInterface for BasicIndex {
    fn add_tokens(&mut self, doc_id: u32, text_to_add: String, mut word_pos: u32) -> u32 {
        for token in text_to_add.split(" ") {
            self.add_posting(token.to_string(), doc_id, word_pos);
            *self.doc_freq.entry(token.to_string()).or_insert(0) += 1;
            word_pos += 1;
        }
        return word_pos;
    }

    fn add_posting(&mut self, token: String, docid: u32, word_pos: u32) {
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

    fn add_document(&mut self, document: Document) {
        let mut word_pos = 0;

        //Metadata
        self.add_document_metadata(
            document.doc_id,
            document.title,
            document.last_updated_date,
            document.namespace,
        );

        //Abstracts
        self.add_abstract(document.doc_id, document.article_abstract);

        // //Infobox

        word_pos = self.add_infoboxes(
            document.doc_id,
            document.infobox_type,
            document.infobox_text,
            document.infobox_ids,
            word_pos,
        );

        //Main body
        word_pos = self.add_main_text(document.doc_id, &document.main_text, word_pos);

        //Citations
        word_pos = self.add_citations(
            document.doc_id,
            document.citations_text,
            document.citations_ids,
            word_pos,
        );

        //Categories
        word_pos = self.add_categories(document.doc_id, document.categories, word_pos);

        //Links
        self.add_links(document.doc_id, &document.article_links);
    }

    fn set_dump_id(&mut self, new_dump_id: u32) {
        self.dump_id = Some(new_dump_id);
    }

    fn add_document_metadata(
        &mut self,
        doc_id: u32,
        title: String,
        last_updated_date: String,
        namespace: u32,
    ) {
        self.document_metadata.insert(
            doc_id,
            DocumentMetaData {
                title,
                last_updated_date,
                namespace,
            },
        );
    }

    fn add_abstract(&mut self, doc_id: u32, article_abstract: String) {
        self.abstracts.insert(doc_id, article_abstract);
    }

    fn add_links(&mut self, doc_id: u32, article_links: &str) {
        let mut link_titles: Vec<String> = Vec::new();
        for link in article_links.split(",") {
            link_titles.push(link.trim().to_string());
        }
        self.links
            .as_mut()
            .expect_left("Index is not in buildable state")
            .insert(doc_id, link_titles);
    }
    fn add_infoboxes(
        &mut self,
        doc_id: u32,
        infobox_type: String,
        text: Vec<String>,
        infobox_ids: Vec<u32>,
        mut word_pos: u32,
    ) -> u32 {
        //TODO: MAKE SURE len(infobox_ids) == len(text)
        for (text_str, infobox_id) in text.iter().zip(infobox_ids.iter()) {
            let prev_pos = word_pos;
            word_pos = self.add_tokens(doc_id, text_str.to_string(), word_pos);
            let extent = ExtentPosting {
                attributed_id: *infobox_id,
                position_start: prev_pos,
                position_end: word_pos,
            };
            self.infoboxes
                .entry(doc_id)
                .or_insert(Vec::<ExtentInfoBox>::new())
                .push(ExtentInfoBox {
                    infobox_positions: extent,
                    infobox_type: infobox_type.to_string(),
                });
        }
        return word_pos;
    }

    fn add_main_text(&mut self, doc_id: u32, main_text: &str, mut word_pos: u32) -> u32 {
        word_pos = self.add_tokens(doc_id, main_text.to_string(), word_pos);
        return word_pos;
    }
    fn add_citations(
        &mut self,
        doc_id: u32,
        citations_body: Vec<String>,
        citation_ids: Vec<u32>,
        mut word_pos: u32,
    ) -> u32 {
        for (text_str, id) in citations_body.iter().zip(citation_ids.iter()) {
            let prev_pos = word_pos;
            word_pos = self.add_tokens(doc_id, text_str.to_string(), word_pos);
            let extent = ExtentPosting {
                attributed_id: *id,
                position_start: prev_pos,
                position_end: word_pos,
            };
            self.citations
                .entry(doc_id)
                .or_insert(Vec::<ExtentCitations>::new())
                .push(ExtentCitations {
                    citation_positions: extent,
                });
        }
        return word_pos;
    }
    //categories_str is a list of categories represented in a string.
    fn add_categories(&mut self, doc_id: u32, categories_str: String, mut word_pos: u32) -> u32 {
        //Parse the query and retrieve the categories
        for cat in categories_str.split(",") {
            let prev_pos = word_pos;
            word_pos = self.add_tokens(doc_id, cat.to_string(), word_pos);

            let extent = ExtentPostingPositionsOnly {
                position_start: prev_pos,
                position_end: word_pos,
            };
            self.categories
                .entry(doc_id)
                .or_insert(Vec::<ExtentCategories>::new())
                .push(ExtentCategories {
                    categories_positions: extent,
                });
        }
        return word_pos;
    }
}
