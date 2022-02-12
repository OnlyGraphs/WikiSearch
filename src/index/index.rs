use crate::index::index_structs::*;
use bimap::BiMap;
use std::collections::HashSet;

use async_trait::async_trait;
use either::{Either, Left, Right};
use std::{
    collections::HashMap,
    fmt,
    marker::{Send, Sync},
};

/**
 * BasicIndex Structure:
 * Metadata <NOT included in postings>
 * Abstracts <NOT included in postings>
 * Infobox <Word position starts here at 0>
 * Main body
 * Citations
 * Categories
 */

pub enum IndexEncoding {
    None,
    DeltaEncoding,
    EliasGammaCode,
}

//TODO: Interface to specify functions that should be shared among different types of indices created (Ternary Index Tree vs BasicIndex)
pub trait Index: Send + Sync {
    fn add_document(&mut self, document: Box<Document>);
    fn set_dump_id(&mut self, new_dump_id: u32);
    fn get_dump_id(self) -> u32;
    fn get_postings(&mut self, token: &str) -> Option<&Vec<Posting>>;
    fn get_extent_for(&mut self, itype: &str, doc_id: &u32) -> Option<&PosRange>;
    fn df(&mut self, token: &str) -> u32;
    fn tf(&mut self, token: &str, docid: u32) -> u32;
    fn finalize(&mut self);

    fn get_links(&mut self, source: u32) -> Vec<u32>;
    fn id_to_title(&mut self, source: u32) -> Option<&String>;
    fn title_to_id(&mut self, source: String) -> Option<u32>;
}
impl fmt::Debug for dyn Index {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//TODO:
//Make sure you check for integer overflows. Or, implementing Delta encoding would mitigate any such problems.

pub struct BasicIndex {
    pub dump_id: Option<u32>,
    pub document_metadata: HashMap<u32, DocumentMetaData>,
    pub postings: HashMap<String, Vec<Posting>>,
    pub doc_freq: HashMap<String, u32>,
    pub term_freq: HashMap<String, HashMap<u32, u32>>, // tf(doc,term) -> frequency in document
    pub links: Either<HashMap<u32, Vec<String>>, HashMap<u32, Vec<u32>>>,
    pub extent: HashMap<String, HashMap<u32, PosRange>>, // structure type -> docid -> pos range
    pub id_title_map: BiMap<u32, String>,
}

impl fmt::Debug for BasicIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "hello")
    }
}

impl Default for BasicIndex {
    fn default() -> Self {
        BasicIndex {
            dump_id: None,
            postings: HashMap::new(),
            doc_freq: HashMap::new(),
            links: Left(HashMap::new()),
            term_freq: HashMap::new(),
            document_metadata: HashMap::new(),
            extent: HashMap::new(),
            id_title_map: BiMap::new(),
        }
    }
}

impl Index for BasicIndex {
    fn get_links(&mut self, source: u32) -> Vec<u32> {
        match self
            .links
            .as_mut()
            .expect_right("Index was not finalized.")
            .get(&source)
        {
            Some(v) => v.clone(),
            None => return Vec::new(),
        }
    }

    fn id_to_title(&mut self, source: u32) -> Option<&String> {
        self.id_title_map.get_by_left(&source)
    }

    fn title_to_id(&mut self, source: String) -> Option<u32> {
        self.id_title_map.get_by_right(&source).map(|c| *c)
    }

    fn finalize(&mut self) {
        // calculate df
        for (token, postings) in self.postings.iter() {
            let mut unique_docs: HashSet<u32> = HashSet::new();

            for Posting { document_id, .. } in postings {
                unique_docs.insert(*document_id);
            }

            self.doc_freq
                .insert(token.to_string(), unique_docs.len() as u32);
        }

        // work out links
        let mut id_links: HashMap<u32, Vec<u32>> = HashMap::new();

        for (id, links) in self.links.as_ref().unwrap_left() {
            let mut targets: Vec<u32> = Vec::new();
            for l in links {
                if let Some(v) = self.id_title_map.get_by_right(l) {
                    targets.push(*v);
                }
            }

            let _ = id_links.insert(*id, targets);
        }
        self.links = Right(id_links);
    }

    fn df(&mut self, token: &str) -> u32 {
        *self.doc_freq.get(token).unwrap_or(&0)
    }

    fn tf(&mut self, token: &str, docid: u32) -> u32 {
        match self.term_freq.get(token) {
            Some(v) => *v.get(&docid).unwrap_or(&0),
            None => return 0,
        }
    }

    fn get_postings(&mut self, token: &str) -> Option<&Vec<Posting>> {
        self.postings.get(token)
    }

    fn get_extent_for(&mut self, itype: &str, doc_id: &u32) -> Option<&PosRange> {
        self.extent.get(itype).and_then(|r| r.get(doc_id))
    }

    fn set_dump_id(&mut self, new_dump_id: u32) {
        self.dump_id = Some(new_dump_id);
    }

    fn get_dump_id(self) -> u32 {
        return self.dump_id.unwrap().clone();
    }

    fn add_document(&mut self, document: Box<Document>) {
        let mut word_pos = 0;

        self.id_title_map
            .insert_no_overwrite(document.doc_id, document.title.clone())
            .expect("Could not insert id-title pair.");

        //Metadata
        self.add_document_metadata(
            document.doc_id,
            document.title,
            document.last_updated_date,
            document.namespace,
        );

        //Infoboxes

        for i in document.infoboxes {
            word_pos = self.add_structure_elem(document.doc_id, &i.itype, i.text, word_pos);
        }

        //Main body
        word_pos = self.add_main_text(document.doc_id, &document.main_text, word_pos);

        //Citations
        for c in document.citations {
            word_pos = self.add_structure_elem(document.doc_id, "citation", c.text, word_pos);
        }

        //Categories
        word_pos =
            self.add_structure_elem(document.doc_id, "categories", document.categories, word_pos);

        //Links
        self.add_links(document.doc_id, &document.article_links);
    }
}

impl BasicIndex {
    fn add_tokens(&mut self, doc_id: u32, text_to_add: String, mut word_pos: u32) -> u32 {
        for token in text_to_add.split(" ") {
            self.add_posting(token.to_string(), doc_id, word_pos);
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

    fn add_document_metadata(
        &mut self,
        doc_id: u32,
        title: String,
        last_updated_date: String,
        namespace: i16,
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

    fn add_structure_elem(
        &mut self,
        doc_id: u32,
        structure_elem: &str,
        text: String,
        mut word_pos: u32,
    ) -> u32 {
        let prev_pos = word_pos;
        word_pos = self.add_tokens(doc_id, text, word_pos);

        self.extent
            .entry(structure_elem.to_string())
            .or_insert(HashMap::new())
            .entry(doc_id)
            .or_insert(PosRange {
                start_pos: prev_pos, // if not exists, initialize range
                end_pos: word_pos,
            })
            .end_pos = word_pos; // if exists, extend it

        return word_pos;
    }

    fn add_main_text(&mut self, doc_id: u32, main_text: &str, mut word_pos: u32) -> u32 {
        word_pos = self.add_tokens(doc_id, main_text.to_string(), word_pos);
        return word_pos;
    }
}
