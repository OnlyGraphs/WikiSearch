use std::collections::{HashMap, HashSet};
use bimap::BiMap;
use parser::StructureElem;
use crate::{PosRange, DiskHashMap, PostingNode, Document, IndexError, IndexErrorKind, Posting};



/// a common backbone from which any index can be intialized
#[derive(Default)]
pub struct PreIndex {
    pub dump_id: u32,
    pub posting_nodes: DiskHashMap<String,PostingNode,1000>,
    pub links: HashMap<u32, Vec<String>>,
    pub extent: HashMap<String, HashMap<u32, PosRange>>,
    pub id_title_map: BiMap<u32, String>,

    // for keeping track of unique token appearances in the current document
    curr_doc_appearances: HashSet<String>
}

impl PreIndex {

    pub fn finalize(){
        
    }

    pub fn add_document(&mut self, document: Box<Document>) -> Result<(), IndexError>{

        let mut word_pos = 0;

        self.id_title_map
            .insert_no_overwrite(document.doc_id, document.title.clone())
            .map_err(|_c| IndexError {
                msg: "Attempted to insert document into index which already exists.".to_string(),
                kind: IndexErrorKind::InvalidOperation,
            })?;

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
        for s in self.curr_doc_appearances.drain(){
            self.posting_nodes.get_mut(&s).unwrap().unwrap().df += 1;
        };

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
        let node = self.posting_nodes.get_or_insert_default_mut(token.to_string()).unwrap();
        self.curr_doc_appearances.insert(token.to_owned());

        node.postings.push(Posting {
            document_id: docid,
            position: word_pos,
        });

        *node.tf.entry(docid).or_default() += 1;
    }

    fn add_links(&mut self, doc_id: u32, article_links: &str) -> Result<(), IndexError> {
        // out links
        let link_titles: Vec<String> = article_links
            .split(",")
            .map(|c| c.trim().to_string())
            .collect();

        // in links
        self.links
            .insert(doc_id, link_titles);
        

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
