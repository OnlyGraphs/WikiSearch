use crate::ast_parser::enums::OpNodeType;
use crate::ast_parser::enums::CategoryNodeType;


// Constants
const CATEGORY_TAG: &str = "#CATEGORY";
const TITLE_TAG: &str = "#TITLE";
const CITATION_TAG: &str = "#CITATION";
const INFO_BOX_TAG: &str = "#INFO_BOX_TEMPLATE_NAME";
const DIST_TAG: &str = "#DIST";
const LINKED_TO_TAG: &str = "#LINKEDTOTAG";
const OR: &str = "OR";
const AND: &str = "AND";
const NOT: &str = "NOT";

pub struct Arena {
    nodes: Vec<Node>,
}

pub struct Node {
    parent: Option<NodeId>,
    previous_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
    first_child: Option<NodeId>,
    last_child: Option<NodeId>,

    /// The actual data which will be stored within the tree
    pub data: String,
}

pub struct CategoryNode {
    node_type: CategoryNodeType,
    parent: Option<NodeId>,
    previous_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
    first_child: Option<NodeId>
}

pub struct OpNode{
    node_type: OpNodeType,
    parent: Option<NodeId>,
    previous_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
    first_child: Option<NodeId>,
    last_child: Option<NodeId>,
}

pub struct DistNode{
    parent: Option<NodeId>,
    previous_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
    first_child: Option<NodeId>,
    last_child: Option<NodeId>,
    
    /// Instead of data, we store the desired distance
    pub dist: u32,
}

pub struct NodeId {
    index: usize,
}

pub fn add_leaf(data: String) -> Node {
    Node {
        data : data,
        first_child : None,
        last_child : None,
        next_sibling : None,
        parent : None,
        previous_sibling : None,
    }
}

pub fn parse_query_to_tree(query: &str) {
    // Create an arena that the tree lives in
    let arena = Arena {
        nodes : Vec::new()
    };
}