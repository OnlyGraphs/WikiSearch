//TODO: Check whether some fields can be set to private

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
pub struct ExtentPosting {
    pub attributed_id: u32, //Refers to citation_id, infobox_id, etc..
    pub position_start: u32,
    pub position_end: u32,
}

#[derive(Debug)]
pub struct ExtentPostingPositionsOnly {
    pub position_start: u32,
    pub position_end: u32,
}
#[derive(Debug)]
pub struct ExtentInfoBox {
    pub infobox_positions: ExtentPosting,
    pub infobox_type: String, //TODO: Could change to this field to enum and define the list of possible infobox types somewhere
}

#[derive(Debug)]
pub struct ExtentCitations {
    pub citation_positions: ExtentPosting,
}
#[derive(Debug)]
pub struct ExtentCategories {
    pub categories_positions: ExtentPostingPositionsOnly,
}

pub struct Document {
    pub doc_id: u32,
    pub title: String,
    pub categories: String,
    pub last_updated_date: String,
    pub namespace: u32,
    pub article_abstract: String,
    pub infobox_type: String,
    pub infobox_text: Vec<String>,
    pub infobox_ids: Vec<u32>,
    pub main_text: String,
    pub article_links: String,
    pub citations_text: Vec<String>,
    pub citations_ids: Vec<u32>,
}

#[derive(Debug)]
pub struct DocumentMetaData {
    pub title: String, //TODO: Implement another field with doc_id -> title and title -> doc_id
    pub last_updated_date: String, //TODO: Change to DateTime type using chrono
    pub namespace: u32, //TODO: Could change this field to enum
}
//TODO:
impl DocumentMetaData {
    fn update_date(&mut self) {}
}