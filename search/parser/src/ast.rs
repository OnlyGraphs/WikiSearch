use std::fmt::{self, Display};
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
        let lowercase = i.to_lowercase();
        match lowercase.as_str() {
            "title" => StructureElem::Title,
            "category" => StructureElem::Category,
            "citation" => StructureElem::Citation,
            _ => StructureElem::Infobox(lowercase.to_string()),
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
        root: u32,
        hops: u8,
        sub: Option<Box<Query>>,
    },
    WildcardQuery {
        prefix: String, // before wildcard
        suffix: String, // after wildcard
    },
    FreetextQuery {
        tokens: Vec<String>,
    },
}

impl Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BinaryOp::And => write!(f, "AND"),
            BinaryOp::Or => write!(f, "OR"),
        }
    }
}

impl Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            UnaryOp::Not => write!(f, "NOT"),
        }
    }
}

impl Display for StructureElem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StructureElem::Title => write!(f, "#TITLE"),
            StructureElem::Category => write!(f, "#CATEGORY"),
            StructureElem::Citation => write!(f, "#CITATION"),
            StructureElem::Infobox(str) => write!(f, "{}", str),
        }
    }
}

impl Display for Query {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Query::BinaryQuery { op, lhs, rhs } => write!(f, "{} {} {}", lhs, op, rhs),
            Query::UnaryQuery { op, sub } => write!(f, "{} {}", op, sub),
            Query::PhraseQuery { tks } => write!(f, "{}", tks.join(" ")),
            Query::DistanceQuery { dst, lhs, rhs } => {
                write!(f, "{},{},{},{}", crate::parser::DIST_TAG, dst, lhs, rhs)
            }
            Query::StructureQuery { elem, sub } => write!(f, "{}: {}", elem, sub),
            Query::RelationQuery { root, hops, sub } => {
                write!(f, "root:{}\n hops:{}\n query:{:?}", root, hops, sub) //TODO: Probably need to do this in a better way
            }
            Query::WildcardQuery { prefix, suffix } => write!(f, "{}*{}", prefix, suffix),
            Query::FreetextQuery { tokens } => {
                write!(f, "{}", tokens.join(" "))
            }
        }
    }
}
