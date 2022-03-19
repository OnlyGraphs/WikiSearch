use log::info;

use utils::MemFootprintCalculator;

use std::env;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Instant;
use std::{collections::HashMap, fmt};

use crate::DiskHashMap;

use crate::EncodedPostingNode;

use crate::index_structs::PosRange;
use crate::Entry;
use crate::LastUpdatedDate;
use crate::Posting;
use crate::VbyteEncoder;

use crate::compute_page_ranks;
use crate::PreIndex;
use parking_lot::Mutex;

#[derive(Default)]
pub struct Index {
    pub dump_id: u32,
    pub posting_nodes: DiskHashMap<String, EncodedPostingNode<VbyteEncoder<Posting,true>>, 0>, // index map because we want to keep this sorted
    pub links: HashMap<u32, Vec<u32>>,
    pub incoming_links: HashMap<u32, Vec<u32>>,
    pub extent: HashMap<String, HashMap<u32, PosRange>>,
    pub last_updated_docs: HashMap<u32, LastUpdatedDate>,
    pub page_rank: HashMap<u32, f64>,
}

impl MemFootprintCalculator for Index {
    fn real_mem(&self) -> u64 {
        self.dump_id.real_mem()
            + self.posting_nodes.real_mem()
            + self.links.real_mem()
            + self.extent.real_mem()
    }
}

impl Debug for Index {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // split calculation to avoid recalculating

        let posting_mem = self.posting_nodes.real_mem();
        let links_mem = self.links.real_mem();
        let incoming_links_mem = self.incoming_links.real_mem();
        let extent_mem = self.extent.real_mem();
        let last_updated_docs_mem = self.last_updated_docs.real_mem();

        let real_mem = self.dump_id.real_mem()
            + posting_mem
            + links_mem
            + incoming_links_mem
            + extent_mem
            + last_updated_docs_mem;

        let mem = real_mem as f64 / 1000000.0;
        let docs = self.links.len();
        write!(
            f,
            "BasicIndex{{\n\
            \tDump ID={:?}\n\
            \tPostingLists={:?}\n\
            \tPostingLists(RAM)={:?}\n\
            \tPostingRAMCapacity={:?}\n\
            \tDocs={:.3}\n\
            \tRAM={:.3}MB\n\
            \tRAM/Docs={:.3}GB/1Million\n\
            \t{{\n\
            \t\tpostings(in cache):{:.3}Mb\n\
            \t\tlinks:{:.3}Mb\n\
            \t\textent:{:.3}Mb\n\
            \t\tmetadata:{:.3}Mb\n\
            \t}}\n\
            }}",
            self.dump_id,
            self.posting_nodes.len(),
            self.posting_nodes.cache_population(),
            self.posting_nodes.capacity(),
            docs,
            mem,
            ((mem / 1000.0) / (docs as f64)) * 1000000.0,
            posting_mem as f64 / 1000000.0,
            (links_mem + incoming_links_mem) as f64 / 1000000.0,
            extent_mem as f64 / 1000000.0,
            last_updated_docs_mem as f64 / 1000000.0
        )
    }
}

impl Index {
    pub fn get_incoming_links(&self, source: u32) -> &[u32] {
        match self.incoming_links.get(&source) {
            Some(v) => v,
            None => &[],
        }
    }

    pub fn get_links(&self, source: u32) -> &[u32] {
        match self.links.get(&source) {
            Some(v) => v,
            None => &[],
        }
    }

    pub fn df(&self, token: &str) -> u32 {
        match self.posting_nodes.entry(token) {
            Some(v) => v.deref().lock().get().unwrap().df,
            None => return 0,
        }
    }

    pub fn tf(&self, token: &str, docid: u32) -> u32 {
        match self.posting_nodes.entry(token) {
            Some(v) => v
                .deref()
                .lock()
                .get()
                .unwrap()
                .tf
                .get(&docid)
                .cloned()
                .unwrap_or(0),
            None => 0,
        }
    }

    pub fn get_number_of_documents(&self) -> u32 {
        self.last_updated_docs.len() as u32
    }

    pub fn get_postings(
        &self,
        token: &str,
    ) -> Option<Arc<Mutex<Entry<EncodedPostingNode<VbyteEncoder<Posting,true>>, 0>>>> {
        self.posting_nodes.entry(token)
    }

    pub fn get_extent_for(&self, itype: &str, doc_id: &u32) -> Option<&PosRange> {
        self.extent.get(itype).and_then(|r| r.get(doc_id))
    }

    pub fn get_dump_id(&self) -> u32 {
        return self.dump_id;
    }

    pub fn get_last_updated_date(&self, doc_id: u32) -> Option<LastUpdatedDate> {
        self.last_updated_docs.get(&doc_id).cloned()
    }

    pub fn with_capacity(
        posting_list_mem_limit: u32,
        posting_list_persistent_mem_limit: u32,
        articles: u32,
    ) -> Self {
        Self {
            dump_id: 0,
            posting_nodes: DiskHashMap::new(
                posting_list_mem_limit,
                posting_list_persistent_mem_limit,
                true,
            ),

            links: HashMap::with_capacity(articles as usize),
            incoming_links: HashMap::with_capacity(articles as usize),
            extent: HashMap::with_capacity(256),
            last_updated_docs: HashMap::with_capacity(articles as usize),
            page_rank: HashMap::with_capacity(articles as usize),
        }
    }

    pub fn from_pre_index(mut p: PreIndex) -> Self {
        // extract postings and sort
        let mut timer = Instant::now();

        // sort links before moving
        p.links.values_mut().for_each(|v| v.sort());

        let mut index = Self {
            dump_id: p.dump_id,
            posting_nodes: p.posting_nodes,
            incoming_links: HashMap::with_capacity(p.links.len()),
            page_rank: HashMap::with_capacity(p.links.len()),
            links: p.links,
            extent: p.extent,
            last_updated_docs: p.last_updated_docs,
        };

        index.posting_nodes.set_runtime_mode();

        // back links
        info!("Generating back links");
        timer = Instant::now();
        index.links.iter().for_each(|(source, target)| {
            target.iter().for_each(|v| {
                index.incoming_links.entry(*v).or_default().push(*source);
            })
        });
        index.incoming_links.values_mut().for_each(|v| v.sort());
        info!("Took {}s", timer.elapsed().as_secs());

        info!("Calculating page rank");
        timer = Instant::now();
        index.page_rank = compute_page_ranks(&index.links, &index.incoming_links, 0.85);
        info!("Took {}s", timer.elapsed().as_secs());

        return index;
    }
}
