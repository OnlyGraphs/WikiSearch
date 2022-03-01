
use strum_macros::IntoStaticStr;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BinaryOp {
    And,
    Or,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum UnaryOp {
    Not,
}

#[derive(Debug, PartialEq, Eq, IntoStaticStr, Clone)]
pub enum StructureElem {
    Title,
    Category,
    Citation,
    Infobox(String), // infobox name
}

impl From<&str> for StructureElem {
    fn from(i: &str) -> Self {
        match i.to_lowercase().as_str() {
            "#title" => StructureElem::Title,
            "#category" => StructureElem::Category,
            "#citation" => StructureElem::Citation,
            _ => StructureElem::Infobox(i.to_string()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Query {
    BinaryQuery {
        op: BinaryOp,
        lhs: Box<Query>,
        rhs: Box<Query>,
    },
    UnaryQuery {
        op: UnaryOp,
        sub: Box<Query>,
    },
    PhraseQuery {
        // TODO: can probably just compound over distance queries
        tks: Vec<String>,
    },
    DistanceQuery {
        dst: u32,
        lhs: String,
        rhs: String,
    },
    StructureQuery {
        elem: StructureElem,
        sub: Box<Query>,
    },
    RelationQuery {
        root: String,
        hops: u32,
        sub: Option<Box<Query>>,
    },
    WildcardQuery {
        prefix: String,  // before wildcard
        postfix: String, // after wildcard
    },
    FreetextQuery {
        tokens: Vec<String>,
    },
}