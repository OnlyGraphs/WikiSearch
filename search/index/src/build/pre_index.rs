use crate::{
    DiskHashMap, Document, IndexError, IndexErrorKind, LastUpdatedDate, PosRange, Posting,
    PostingNode, DATE_TIME_FORMAT,
};
use bimap::BiMap;
use chrono::NaiveDateTime;
use itertools::Itertools;
use parser::StructureElem;
use std::collections::{HashMap, HashSet};

/// a common backbone from which any index can be intialized
pub struct PreIndex {
    pub dump_id: u32,
    pub posting_nodes: DiskHashMap<String, PostingNode, 0>,
    pub links: HashMap<u32, Vec<u32>>,
    pub extent: HashMap<String, HashMap<u32, PosRange>>,
    // pub id_title_map: BiMap<u32, String>,
    pub last_updated_docs: HashMap<u32, LastUpdatedDate>,
    // for keeping track of unique token appearances in the current document
    curr_doc_appearances: HashSet<String>,
}

impl Default for PreIndex {
    fn default() -> Self {
        Self {
            dump_id: Default::default(),
            posting_nodes: DiskHashMap::new(10000, true),
            links: Default::default(),
            extent: Default::default(),
            // id_title_map: Default::default(),
            last_updated_docs: Default::default(),
            curr_doc_appearances: Default::default(),
        }
    }
}

impl PreIndex {
    pub fn with_capacity(cap: u32) -> Self {
        Self {
            dump_id: Default::default(),
            posting_nodes: DiskHashMap::new(cap, true),
            links: Default::default(),
            extent: Default::default(),
            // id_title_map: Default::default(),
            last_updated_docs: Default::default(),
            curr_doc_appearances: Default::default(),
        }
    }

    pub fn cache_size(&self) -> u32 {
        self.posting_nodes.cache_population()
    }

    pub fn clean_cache(&self) {
        self.posting_nodes.clean_cache();
    }

    pub fn add_document(&mut self, document: Box<Document>) -> Result<(), IndexError> {
        if self.links.contains_key(&document.doc_id) {
            return Err(IndexError {
                msg: "Attempted to insert document into index which already exists.".to_string(),
                kind: IndexErrorKind::InvalidOperation,
            });
        }

        let mut word_pos = 0;

        // metadata
        self.last_updated_docs.insert(
            document.doc_id,
            LastUpdatedDate {
                date_time: NaiveDateTime::parse_from_str(
                    &document.last_updated_date,
                    DATE_TIME_FORMAT,
                )
                .unwrap_or(NaiveDateTime::from_timestamp(0, 0)),
            },
        );

        // // titles
        // self.id_title_map
        //     .insert_no_overwrite(document.doc_id, document.title.clone())
        //     .map_err(|_c| IndexError {
        //         msg: "Attempted to insert document into index which already exists.".to_string(),
        //         kind: IndexErrorKind::InvalidOperation,
        //     })?;

        //Infoboxes
        word_pos = document.infoboxes.iter().fold(word_pos, |a, i| {
            self.add_structure_elem(document.doc_id, &i.itype, &i.text, a)
        });

        //Main body
        word_pos = self.add_main_text(document.doc_id, &document.main_text, word_pos);

        //Citations
        word_pos = document.citations.iter().fold(word_pos, |a, c| {
            self.add_structure_elem(document.doc_id, StructureElem::Citation.into(), &c.text, a)
        });

        //Categories
        let _ = self.add_structure_elem(
            document.doc_id,
            StructureElem::Category.into(),
            &document.categories,
            word_pos,
        );

        //Links
        self.add_links(document.doc_id, &document.article_links)?;

        // collect DF values
        for s in self.curr_doc_appearances.drain() {
            self.posting_nodes
                .entry(&s)
                .unwrap()
                .lock()
                .get_mut()
                .unwrap()
                .df += 1;
        }

        Ok(())
    }

    fn add_tokens(&mut self, doc_id: u32, text_to_add: &str, mut word_pos: u32) -> u32 {
        for token in text_to_add.split(" ") {
            self.add_posting(token, doc_id, word_pos);
            word_pos += 1;
        }
        return word_pos;
    }

    fn add_posting(&mut self, token: &str, docid: u32, word_pos: u32) {
        let ptr = self.posting_nodes.entry_or_default(token);

        let mut lock = ptr.lock();

        let node = lock.get_mut().unwrap();

        self.curr_doc_appearances.insert(token.to_owned());

        node.postings.push(Posting {
            document_id: docid,
            position: word_pos,
        });

        *node.tf.entry(docid).or_default() += 1;
    }

    fn add_links(&mut self, doc_id: u32, article_links: &str) -> Result<(), IndexError> {
        // out links
        let link_titles: Vec<u32> = article_links
            .split("\t")
            .filter_map(|c| c.trim().to_string().parse::<u32>().ok())
            .collect();

        // in links
        self.links.insert(doc_id, link_titles);

        Ok(())
    }

    fn add_structure_elem(
        &mut self,
        doc_id: u32,
        structure_elem: &str,
        text: &str,
        mut word_pos: u32,
    ) -> u32 {
        let prev_pos = word_pos;
        word_pos = self.add_tokens(doc_id, &text, word_pos);

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
        word_pos = self.add_tokens(doc_id, main_text, word_pos);
        return word_pos;
    }
}
