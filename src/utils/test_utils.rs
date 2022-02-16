use crate::index::index_structs::{Citation, Document, Infobox};

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
        namespace: i16::default(),
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
        namespace: i16::default(),
        article_links: links.to_string(),
    })
}
