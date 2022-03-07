use crate::index_structs::{
    Citation, Document, DocumentMetaData, Infobox, PosRange, Posting, PostingNode,
};
use std::mem::size_of;
use utils::MemFootprintCalculator;

macro_rules! implMemFootprintCalculatorFor {
    ( $($t:ty),* ) => {
    $( impl MemFootprintCalculator for $t{
        fn real_mem(&self) -> u64{
            size_of::<$t>() as u64
        }
    }) *
    }
}

implMemFootprintCalculatorFor!(Posting, PosRange);

impl MemFootprintCalculator for PostingNode {
    fn real_mem(&self) -> u64 {
        self.postings.real_mem() + self.df.real_mem() + self.tf.real_mem()
        // above already counts metadata
    }
}

impl MemFootprintCalculator for DocumentMetaData {
    fn real_mem(&self) -> u64 {
        self.last_updated_date.real_mem() + self.namespace.real_mem() + self.title.real_mem()
        // above already counts metadata
    }
}

#[allow(dead_code)]
pub fn get_document_with_text_and_links(
    id: u32,
    title: &str,
    infoboxes: Vec<(&str, &str)>,
    main_text: &str,
    citations: Vec<&str>,
    categories: &str,
    links: &str,
) -> Box<Document> {
    let mut a = get_document_with_text(id, title, infoboxes, main_text, citations, categories);
    a.article_links = links.to_string();
    return a;
}

#[allow(dead_code)]
pub fn get_document_with_text(
    id: u32,
    title: &str,
    infoboxes: Vec<(&str, &str)>,
    main_text: &str,
    citations: Vec<&str>,
    categories: &str,
) -> Box<Document> {
    Box::new(Document {
        title: title.to_string(),
        doc_id: id,
        infoboxes: infoboxes
            .iter()
            .map(|(a, b)| Infobox {
                itype: a.to_string(),
                text: b.to_string(),
            })
            .collect(),
        main_text: main_text.to_string(),
        citations: citations
            .iter()
            .map(|c| Citation {
                text: c.to_string(),
            })
            .collect(),
        categories: categories.to_string(),
        last_updated_date: String::default(),
        article_links: String::default(),
    })
}

#[allow(dead_code)]
pub fn get_document_with_links(id: u32, title: &str, links: &str) -> Box<Document> {
    Box::new(Document {
        title: title.to_string(),
        doc_id: id,
        infoboxes: vec![],
        main_text: String::default(),
        citations: vec![],
        categories: String::default(),
        last_updated_date: String::default(),
        article_links: links.to_string(),
    })
}

pub fn get_document_with_date_time(id: u32, title: &str, last_updated_date: &str) -> Box<Document> {
    Box::new(Document {
        title: title.to_string(),
        doc_id: id,
        infoboxes: vec![],
        main_text: String::default(),
        citations: vec![],
        categories: String::default(),
        last_updated_date: last_updated_date.to_string(),
        article_links: String::default(),
    })
}
