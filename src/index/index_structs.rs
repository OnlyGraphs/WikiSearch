#[derive(Debug)]
pub enum Domain {
    simple,
}

// stores an appearance of a token in an article
#[derive(Debug)]
pub struct Posting {
    pub document_id: u32, //TODO: double check memory requirements, highest article word count etc
    pub position: u32,
}
#[derive(Debug)]
pub struct PostingRange {
    pub document_id: u32,
    pub position_start: u32,
    pub position_end: u32,
}
#[derive(Debug)]
pub struct InfoBox {
    pub infobox_positions: PostingRange,
    pub infobox_type: String, //TODO: Could change to this field to enum and define the list of possible infobox types somewhere
}

#[derive(Debug)]
pub struct Citations {
    pub citation_positions: PostingRange,
    pub citation_string: String,
}
#[derive(Debug)]
pub struct DocumentMetaData {
    pub title: String,
    pub lastUpdatedDate: String, //TODO: Change to DateTime type using chrono
    pub namespace: u32,          //TODO: Could change this field to enum
}
