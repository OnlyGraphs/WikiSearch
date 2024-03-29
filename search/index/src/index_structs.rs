use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use indexmap::IndexMap;

use crate::{
    DeltaEncoder, EncodedPostingList, EncodedSequentialObject, SequentialEncoder, VbyteEncoder,
};
use fxhash::FxBuildHasher;
use utils::MemFootprintCalculator;
pub const DATE_TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// stores an appearance of a token in an article
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Default)] //TODO: get rid of Copy and correct parts of program which use it
pub struct Posting {
    pub document_id: u32, //TODO: double check memory requirements, highest article word count etc
    pub position: u32,
}

impl Into<(u32, u32)> for Posting {
    fn into(self) -> (u32, u32) {
        (self.document_id, self.position)
    }
}

impl Into<(u32, u32)> for &Posting {
    fn into(self) -> (u32, u32) {
        (self.document_id, self.position)
    }
}

#[derive(Debug, Eq, PartialEq, Default, Clone)]
pub struct PostingNode {
    pub postings: Vec<Posting>,
    pub df: u32,
    pub tf: IndexMap<u32, u32, FxBuildHasher>,
    pub postings_count: u32,
}

#[derive(Debug, Eq, PartialEq, Default, Clone)]
pub struct EncodedPostingNode<E>
where
    E: SequentialEncoder<Posting>,
{
    pub postings: EncodedPostingList<E>,
    pub df: u32,
    pub tf: IndexMap<u32, u32, FxBuildHasher>,
    pub postings_count: u32,
}

#[derive(Default, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PosRange {
    pub start_pos: u32, //TODO: double check memory requirements, highest article word count etc
    pub end_pos: u32,
}

pub struct Infobox {
    pub itype: String,
    pub text: String,
}

pub struct Citation {
    pub text: String,
}

#[derive(Default)]
pub struct Document {
    pub doc_id: u32,
    pub title: String,
    pub categories: String,
    pub last_updated_date: String,
    pub main_text: String,
    pub article_links: String,
    pub infoboxes: Vec<Infobox>,
    pub citations: Vec<Citation>,
}

#[derive(Debug)]
pub struct DocumentMetaData {
    pub title: String, //TODO: Implement another field with doc_id -> title and title -> doc_id
    pub last_updated_date: Option<LastUpdatedDate>, //TODO: Change to DateTime type using chrono
    pub namespace: i16, //TODO: Could change this field to enum
}
#[derive(PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Debug)]
pub struct LastUpdatedDate {
    pub date_time: NaiveDateTime,
}

impl MemFootprintCalculator for LastUpdatedDate {
    fn real_mem(&self) -> u64 {
        self.date_time.real_mem()
    }
}

// impl Default for LastUpdatedDate {
//     fn default() -> Self {
//         let d = NaiveDate::from_ymd(0, 1, 1);
//         let t = NaiveTime::from_hms(0, 0, 0);
//         LastUpdatedDate {
//             date_time: NaiveDateTime::new(d, t),
//         }
//     }
// }
