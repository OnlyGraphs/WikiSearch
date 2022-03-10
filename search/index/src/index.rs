use chrono::NaiveDateTime;
use log::info;
use sqlx::database::HasArguments;
use streaming_iterator::{convert_ref, StreamingIterator,convert};
use utils::MemFootprintCalculator;

use std::fmt::Debug;
use std::fmt::Formatter;
use std::ops::Deref;
use std::sync::Arc;
use std::time::Instant;
use std::{
    collections::HashMap,
    fmt,
};

use crate::DecoderIterator;
use crate::DiskHashMap;
use crate::EncodedPostingList;
use crate::EncodedPostingNode;
use crate::EncodedSequentialObject;
use crate::Entry;
use crate::IdentityEncoder;
use crate::index_structs::{PosRange, Posting};
use crate::PostingNode;
use crate::PreIndex;
use parking_lot::{Mutex, MutexGuard, MappedMutexGuard};
// use crate::init_page_rank;


#[derive(Default)]
pub struct Index{
    pub dump_id: u32,
    pub posting_nodes: DiskHashMap<String, EncodedPostingNode<IdentityEncoder>,1>, // index map because we want to keep this sorted
    pub links: HashMap<u32, Vec<u32>>,
    pub incoming_links: HashMap<u32, Vec<u32>>,
    pub extent: HashMap<String, HashMap<u32, PosRange>>,
    pub last_updated_docs: HashMap<u32, NaiveDateTime>,
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
            \tPostingInRAM={:?}\n\
            \tPostingRamCapacity={:?}\n\
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
            Some(v) => v.deref().lock().get().unwrap().tf
                                                                    .get(&docid).cloned().unwrap_or(0),
            None => 0,
        }
    }

    pub fn get_number_of_documents(&self) -> u32 {
        self.last_updated_docs.len() as u32
    }


    pub fn get_postings(&self, token: &str) -> Option<Arc<Mutex<Entry<EncodedPostingNode<IdentityEncoder>,1>>>>
    {
        self.posting_nodes.entry(token)
    }


    pub fn get_extent_for(&self, itype: &str, doc_id: &u32) -> Option<&PosRange> {
        self.extent.get(itype).and_then(|r| r.get(doc_id))
    }

    pub fn get_dump_id(&self) -> u32 {
        return self.dump_id;
    }

    pub fn get_last_updated_date(&self, doc_id: u32) -> Option<NaiveDateTime> {
        self.last_updated_docs.get(&doc_id).cloned()
    }

    pub fn with_capacity(
        posting_list_mem_limit: u32,
        articles: u32
    ) -> Self {
        Self {
            dump_id: 0,
            posting_nodes: DiskHashMap::new(posting_list_mem_limit),
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

        let total_posting_lists = p.posting_nodes.len();
        info!("Sorting {} posting lists", total_posting_lists);
        let mut posting_nodes : DiskHashMap<String, EncodedPostingNode<IdentityEncoder>,1> = DiskHashMap::new(p.posting_nodes.capacity());

        (0..p.posting_nodes.len()).for_each(|idx| {

            { 
            let (k, v) = p.posting_nodes.pop().unwrap();
            v.lock().get_mut().unwrap().postings.sort();

            let unwrapped = match Arc::try_unwrap(v) {
                Ok(v) => v,
                Err(_) => panic!(),
            }  .into_inner()
                .into_inner()
                .unwrap();

            let encoded_node = EncodedPostingNode::from(unwrapped);
            posting_nodes.insert(k,encoded_node);
            } // for dropping lock on v in case we want to evict it

            // every R records, clean cache, and report progress
            if idx % posting_nodes.capacity() as usize == 0 {
                info!("Sorted {}%, cache pop: {} ({}s)", (idx as f32 / total_posting_lists as f32) * 100.0,posting_nodes.clean_cache(),timer.elapsed().as_secs());
                timer = Instant::now();
                
            }
        });

        posting_nodes.clean_cache();
        

        // convert strings in the links to u32's
        // sort all links
        info!("Reconciling links with IDs");
        timer = Instant::now();
        let mut links: HashMap<u32, Vec<u32>> = HashMap::with_capacity(p.links.len());
        p.links.iter().for_each(|(from, to)| {
            let mut targets: Vec<u32> = Vec::with_capacity(links.len());
            to.iter().for_each(|l| {
                p.id_title_map.get_by_right(l).map(|v| {
                    targets.push(*v);
                });
            });
            targets.sort();
            let _ = links.insert(*from, targets);
        });
        info!("Took {}s", timer.elapsed().as_secs());

        // back links
        info!("Generating back links");
        timer = Instant::now();
        let mut back_links: HashMap<u32, Vec<u32>> = HashMap::with_capacity(p.links.len());
        links.iter().for_each(|(source, target)| {
            target.iter().for_each(|v| {
                back_links.entry(*v).or_default().push(*source);
            })
        });
        back_links.values_mut().for_each(|v| v.sort());
        info!("Took {}s", timer.elapsed().as_secs());

        Self {
            dump_id: p.dump_id,
            posting_nodes: posting_nodes,
            links: links,
            incoming_links: back_links,
            extent: p.extent,
            last_updated_docs: p.last_updated_docs,
            page_rank: HashMap::default(), //init_page_rank(back_links, 1.0),
        }
    }
}
