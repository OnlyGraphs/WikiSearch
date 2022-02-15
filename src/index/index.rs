use bimap::BiMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::fmt::Formatter;

use crate::index::{
    index_structs::{Document, DocumentMetaData, PosRange, Posting, PostingNode},
    errors::{IndexError, IndexErrorKind},
    collections::{StringPostingMap}
};

use crate::utils::utils::MemFootprintCalculator;

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
pub trait Index: Send + Sync + Debug + MemFootprintCalculator {
    fn add_document(&mut self, document: Box<Document>)-> Result<(),IndexError> ;
    fn set_dump_id(&mut self, new_dump_id: u32);
    fn get_dump_id(&self) -> u32;
    fn get_postings(&self, token: &str) -> Option<&[Posting]>;
    fn get_extent_for(&self, itype: &str, doc_id: &u32) -> Option<&PosRange>;
    fn df(&self, token: &str) -> u32;
    fn tf(&self, token: &str, docid: u32) -> u32;
    fn finalize(&mut self) -> Result<(),IndexError>;

    fn get_links(&self, source: u32) -> Result<&[u32],IndexError>;
    fn id_to_title(&self, source: u32) -> Option<&String>;
    fn title_to_id(&self, source: String) -> Option<u32>;
}

//TODO:
//Make sure you check for integer overflows. Or, implementing Delta encoding would mitigate any such problems.

pub struct BasicIndex<M: StringPostingMap + ?Sized> {
    pub dump_id: Option<u32>,
    pub document_metadata: HashMap<u32, DocumentMetaData>,
    pub posting_nodes: M,
    pub links: Either<HashMap<u32, Vec<String>>, HashMap<u32, Vec<u32>>>,
    pub extent: HashMap<String, HashMap<u32, PosRange>>,
    pub id_title_map: BiMap<u32, String>,
}

impl <M : StringPostingMap>  Default for BasicIndex<M> {
    fn default() -> Self {
        BasicIndex{
            dump_id: None,
            posting_nodes: M::default(),
            links: Left(HashMap::new()),
            document_metadata: HashMap::new(),
            extent: HashMap::new(),
            id_title_map: BiMap::new(),
        }
    }
}

impl <M : StringPostingMap> MemFootprintCalculator for BasicIndex<M> {
    fn real_mem(&self) -> u64 {
        self.dump_id.real_mem()
            + self.posting_nodes.real_mem()
            + self.links.real_mem()
            + self.document_metadata.real_mem()
            + self.extent.real_mem()
            + self.id_title_map.real_mem()
    }
}

impl <M : StringPostingMap> Debug for BasicIndex<M> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {

        // split calculation to avoid recalculating
        let posting_mem = self.posting_nodes.real_mem();
        let metadata_mem = self.document_metadata.real_mem();
        let links_mem = self.links.real_mem();
        let extent_mem = self.extent.real_mem();
        let id_map_mem = self.id_title_map.real_mem();

        let real_mem = self.dump_id.real_mem()
                        + posting_mem
                        + links_mem
                        + metadata_mem
                        + extent_mem
                        + id_map_mem;

        let mem = real_mem as f64 / 1000000.0;
        let docs = self.links.as_ref().either(|c| c.len(), |c| c.len());

        write!(
            f,
            "BasicIndex{{\n\
            \tDump ID={:?}\n\
            \tPostings={:?}\n\
            \tDocs={:.3}\n\
            \tRAM={:.3}MB\n\
            \tRAM/Docs={:.3}GB/1Million\n\
            \t{{\n\
            \t\tpostings:{:.3}Mb\n\
            \t\tmetadata:{:.3}Mb\n\
            \t\tlinks:{:.3}Mb\n\
            \t\textent:{:.3}Mb\n\
            \t\ttitles:{:.3}Mb\n\
            \t}}\n\
            }}",
            self.dump_id,
            self.posting_nodes.len(),
            docs,
            mem,
            ((mem / 1000.0) / (docs as f64)) * 1000000.0,
            posting_mem as f64 / 1000000.0,
            metadata_mem as f64 / 1000000.0,
            links_mem as f64 / 1000000.0,
            extent_mem as f64 / 1000000.0,
            id_map_mem as f64 / 1000000.0,
        )
    }
}

impl <M : StringPostingMap> Index for BasicIndex<M> {

    fn get_links(&self, source: u32) -> Result<&[u32],IndexError> {
        match self
            .links
            .as_ref()
            .right()
            .ok_or(IndexError{
                msg: "Cannot retrieve links, index was not finalized.".to_string(),
                kind: IndexErrorKind::InvalidIndexState,
            })?
            .get(&source)
        {
            Some(v) => Ok(v),
            None => Ok(&[]),
        }
    }

    fn id_to_title(&self, source: u32) -> Option<&String> {
        self.id_title_map.get_by_left(&source)
    }

    fn title_to_id(&self, source: String) -> Option<u32> {
        self.id_title_map.get_by_right(&source).cloned()
    }

    fn finalize(&mut self) -> Result<(),IndexError> {
        // calculate df
        for (_token, postings) in self.posting_nodes.iter_mut() {
            let mut unique_docs: HashSet<u32> = HashSet::with_capacity(self.id_title_map.len());

            for Posting { document_id, .. } in &postings.postings {
                unique_docs.insert(*document_id);
            }

            postings.df = unique_docs.len() as u32;
        }

        // work out links
        let mut id_links: HashMap<u32, Vec<u32>> = HashMap::with_capacity(self.id_title_map.len());

        for (id, links) in self.links.as_ref().left().ok_or(
            IndexError {
                msg: "Index was already finalized, cannot finalize again.".to_string(),
                kind: IndexErrorKind::InvalidIndexState
            })? 
        {
            let mut targets: Vec<u32> = Vec::with_capacity(links.len());
            for l in links {
                if let Some(v) = self.id_title_map.get_by_right(l) {
                    targets.push(*v);
                }
            }

            let _ = id_links.insert(*id, targets);
        }
        self.links = Right(id_links);

        // sort postings
        self.posting_nodes.iter_mut().for_each(|(_k,v)| v.postings.sort());

        Ok(())
    }

    fn df(&self, token: &str) -> u32 {
        match self.posting_nodes.get(token) {
            Some(v) => v.df,
            None => return 0,
        }
    }

    fn tf(&self, token: &str, docid: u32) -> u32 {
        match self.posting_nodes.get(token) {
            Some(v) => v.tf.get(&docid).cloned().unwrap_or(0),
            None => 0,
        }
    }

    fn get_postings(&self, token: &str) -> Option<&[Posting]> {
        self.posting_nodes
            .get(token)
            .and_then(|c| Some(c.postings.as_slice()))
    }

    fn get_extent_for(&self, itype: &str, doc_id: &u32) -> Option<&PosRange> {
        self.extent.get(itype).and_then(|r| r.get(doc_id))
    }

    fn set_dump_id(&mut self, new_dump_id: u32) {
        self.dump_id = Some(new_dump_id);
    }

    fn get_dump_id(&self) -> u32 {
        return self.dump_id.unwrap_or(0).clone();
    }

    fn add_document(&mut self, document: Box<Document>) -> Result<(),IndexError> {
        let mut word_pos = 0;

        self.id_title_map
            .insert_no_overwrite(document.doc_id, document.title.clone())
            .map_err(|_c| IndexError{
                msg: "Attempted to insert document into index which already exists.".to_string(),
                kind: IndexErrorKind::InvalidOperation,
            })?;

        //Metadata
        self.add_document_metadata(
            document.doc_id,
            document.title,
            document.last_updated_date,
            document.namespace,
        );

        //Infoboxes
        word_pos = document.infoboxes.iter()
            .fold(word_pos,|a,i| self.add_structure_elem(document.doc_id, &i.itype, &i.text, a));

        //Main body
        word_pos = self.add_main_text(document.doc_id, &document.main_text, word_pos);

        //Citations
        word_pos = document.citations.iter()
            .fold(word_pos,|a,c| self.add_structure_elem(document.doc_id, "citation", &c.text, a));

        //Categories
        let _ = self.add_structure_elem(
            document.doc_id,
            "categories",
            &document.categories,
            word_pos,
        );

        //Links
        self.add_links(document.doc_id, &document.article_links)?;

        Ok(())
    }
}

impl <M : StringPostingMap>BasicIndex<M> {
    pub fn with_capacity(articles:usize, avg_tokens_per_article: usize, struct_elem_type_count: usize) -> Box<Self>{
        Box::new(
            BasicIndex{
                dump_id: None,
                posting_nodes: M::with_capacity(articles * avg_tokens_per_article) ,
                links: Left(HashMap::with_capacity(articles )),
                document_metadata: HashMap::with_capacity(articles),
                extent: HashMap::with_capacity(struct_elem_type_count),
                id_title_map: BiMap::with_capacity(articles),
            }
        )
    }

    fn add_tokens(&mut self, doc_id: u32, text_to_add: &str, mut word_pos: u32) -> u32 {
        for token in text_to_add.split(" ") {
            self.add_posting(token, doc_id, word_pos);
            word_pos += 1;
        }
        return word_pos;
    }

    fn add_posting(&mut self, token: &str, docid: u32, word_pos: u32) {
        let node = self.posting_nodes.entry(token.to_string()).or_default();

        node.postings.push(Posting {
            document_id: docid,
            position: word_pos,
        });

        *node.tf.entry(docid).or_default() += 1;
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

    fn add_links(&mut self, doc_id: u32, article_links: &str) -> Result<(),IndexError> {
        let link_titles : Vec<String> = article_links
            .split(",")
            .map(|c| c.trim().to_string())
            .collect();

        self.links
            .as_mut()
            .left()
            .ok_or(IndexError{
                msg: "Attempted to add links to already finalized index.".to_string(),
                kind: IndexErrorKind::InvalidIndexState
            })?
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
