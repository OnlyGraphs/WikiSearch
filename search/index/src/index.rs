use chrono::NaiveDateTime;
use utils::MemFootprintCalculator;





use std::fmt::Debug;
use std::fmt::Formatter;
use std::{
    collections::HashMap,
    fmt,
    marker::{Send, Sync},
};

use crate::PostingNode;
use crate::PreIndex;
use crate::{
    index_structs::{PosRange, Posting},
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

//TODO: Interface to specify functions that should be shared among different types of indices created (Ternary Index Tree vs BasicIndex)
pub trait Index: Send + Sync + Debug + MemFootprintCalculator {
    fn get_dump_id(&self) -> u32;
    fn get_postings(&self, token: &str) -> Option<&[Posting]>;
    fn get_all_postings(&self) -> Vec<Posting>;

    fn get_extent_for(&self, itype: &str, doc_id: &u32) -> Option<&PosRange>;
    fn df(&self, token: &str) -> u32;
    fn tf(&self, token: &str, docid: u32) -> u32;
    fn get_number_of_documents(&self) -> u32;

    fn get_links(&self, source: u32) -> &[u32];
    fn get_incoming_links(&self, source: u32) -> &[u32];
    fn get_last_updated_date(&self, source: u32) -> Option<NaiveDateTime>;
}

//TODO:
//Make sure you check for integer overflows. Or, implementing Delta encoding would mitigate any such problems.

#[derive(Default)]
pub struct BasicIndex {
    pub dump_id: u32,
    pub posting_nodes: HashMap<String, PostingNode>,
    pub links: HashMap<u32, Vec<u32>>,
    pub incoming_links: HashMap<u32, Vec<u32>>,
    pub extent: HashMap<String, HashMap<u32, PosRange>>,
    pub last_updated_docs: HashMap<u32, NaiveDateTime>,
}


impl MemFootprintCalculator for BasicIndex {
    fn real_mem(&self) -> u64 {
        self.dump_id.real_mem()
            + self.posting_nodes.real_mem()
            + self.links.real_mem()
            + self.extent.real_mem()
    }
}

impl Debug for BasicIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // split calculation to avoid recalculating
        let posting_mem = self.posting_nodes.real_mem();
        let links_mem = self.links.real_mem();
        let extent_mem = self.extent.real_mem();

        let real_mem = self.dump_id.real_mem()
            + posting_mem
            + links_mem
            + extent_mem;

        let mem = real_mem as f64 / 1000000.0;
        let docs = self.links.len();

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
            \t\tlinks:{:.3}Mb\n\
            \t\textent:{:.3}Mb\n\
            \t}}\n\
            }}",
            self.dump_id,
            self.posting_nodes.len(),
            docs,
            mem,
            ((mem / 1000.0) / (docs as f64)) * 1000000.0,
            posting_mem as f64 / 1000000.0,
            links_mem as f64 / 1000000.0,
            extent_mem as f64 / 1000000.0,
        )
    }
}

impl Index for BasicIndex {
    fn get_incoming_links(&self, source: u32) -> &[u32] {
        match self.incoming_links.get(&source) {
            Some(v) => v,
            None => &[],
        }
    }

    fn get_links(&self, source: u32) -> &[u32] {
        match self.links.get(&source)
        {
            Some(v) => v,
            None => &[],
        }
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

    fn get_number_of_documents(&self) -> u32 {
        return 0; //self.id_title_map.len() as u32;
    }

    // TODO: some sort of batching wrapper over postings lists, to later support lists of postings bigger than memory
    fn get_postings(&self, token: &str) -> Option<&[Posting]> {
        self.posting_nodes
            .get(token)
            .and_then(|c| Some(c.postings.as_slice()))
    }

    // TODO: some sort of batching wrapper over postings lists, to later support lists of postings bigger than memory
    fn get_all_postings(&self) -> Vec<Posting> {
        let mut out = self.posting_nodes
            .iter()
            .flat_map(|(_, v)| v.postings.clone())
            .collect::<Vec<Posting>>();
        out.sort();

        return out;
    }

    fn get_extent_for(&self, itype: &str, doc_id: &u32) -> Option<&PosRange> {
        self.extent.get(itype).and_then(|r| r.get(doc_id))
    }


    fn get_dump_id(&self) -> u32 {
        return self.dump_id;
    }

    fn get_last_updated_date(&self, doc_id: u32) -> Option<NaiveDateTime> {
        self.last_updated_docs.get(&doc_id).cloned()
    }

    
}

impl BasicIndex {
    pub fn with_capacity(
        articles: usize,
        avg_tokens_per_article: usize,
        struct_elem_type_count: usize,
    ) -> Box<Self> {
        Box::new(BasicIndex {
            dump_id: 0,
            posting_nodes: HashMap::with_capacity(articles * avg_tokens_per_article),
            links: HashMap::with_capacity(articles),
            incoming_links: HashMap::with_capacity(articles),
            extent: HashMap::with_capacity(struct_elem_type_count),
            last_updated_docs: HashMap::with_capacity(articles),
        })
    }

    pub fn from_pre_index(mut p : PreIndex) -> Box<Self> {

        // extract postings and sort
        let mut posting_nodes = HashMap::with_capacity(p.posting_nodes.len());
        p.posting_nodes.iter_idx().for_each(|_| {
            // we do not use get_by_idx, since the indexes will change, we only care about what's next in order
            let (k,mut v) = p.posting_nodes.remove_first().unwrap();
            v.postings.sort();
            posting_nodes.insert(k,v);
        });

        // convert strings in the links to u32's
        // sort all links
        let mut links: HashMap<u32, Vec<u32>> = HashMap::with_capacity(p.links.len());
        p.links.iter().for_each(|(from,to)| {
            let mut targets: Vec<u32> = Vec::with_capacity(links.len());
            to.iter().for_each(|l| {
                p.id_title_map.get_by_right(l).map(|v| {
                    targets.push(*v);
                });
            });
            targets.sort();
            let _ = links.insert(*from, targets);
        });

        // back links
        let mut back_links: HashMap<u32, Vec<u32>> = HashMap::with_capacity(p.links.len());
        links.iter().for_each(|(source,target)| {
            target.iter().for_each(|v| {
                back_links.entry(*v).or_default().push(*source);
            })
        });
        back_links.values_mut().for_each(|v| v.sort());

        Box::new(
            BasicIndex{
                dump_id: p.dump_id,
                posting_nodes: posting_nodes,
                links: links,
                incoming_links: back_links,
                extent: p.extent,
                last_updated_docs: p.last_updated_docs,
            }
        )

        
    }
}
