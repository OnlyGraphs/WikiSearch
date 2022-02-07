
pub enum BinaryOp {
    And,
    Or
}

pub enum UnaryOp {
    Not,
}

pub enum StructElem{
    Title,
    Category,
    Citation,
    Infobox(String) // infobox name
}

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
    PhraseQuery { // TODO: can probably just compound over distance queries
        tks: Vec<String>,
    },
    DistanceQuery {
        dst: i32,
        lhs: String,
        rhs: String,
    },
    StructureQuery {
        elem: StructElem,
        sub: Box<Query>,
    },
    RelationQuery {
        root: String,
        hops: u32,
        sub: Box<Option<Query>>,
    },
    WildcardQuery {
        prefix: String, // before wildcard
        postfix: String, // after wildcard
    },
}