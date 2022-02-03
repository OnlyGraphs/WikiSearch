mod enums

// Tree implementation, taken from https://rust-leipzig.github.io/architecture/2016/12/20/idiomatic-trees-in-rust/

pub struct Arena<T> {
    nodes: Vec<Node<T>>,
}

pub struct Node<T> {
    type: enums::NodeType,
    parent: Option<NodeId>,
    previous_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
    first_child: Option<NodeId>,
    last_child: Option<NodeId>,

    /// The actual data which will be stored within the tree
    pub data: vec<String>,
}

pub struct CategoryNode<T> {
    type: enums::CategoryNodeType,
    parent: Option<NodeId>,
    previous_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
    first_child: Option<NodeId>
}

pub struct OpNode<T>{
    type: enums::OpNodeType.
    parent: Option<NodeId>,
    previous_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
    first_child: Option<NodeId>,
    last_child: Option<NodeId>,
}

pub struct DistNode<T>{
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

pub fn new_node(&mut self, data: vec<String>, node_type: enums::NodeType) -> NodeId {
    // Get the next free index
    let next_index = self.nodes.len();

    // Push the node into the arena
    self.nodes.push(Node {
        type: NodeType
        parent: None,
        first_child: None,
        last_child: None,
        previous_sibling: None,
        next_sibling: None,
        data: data,
    });

    // Return the node identifier
    NodeId { index: next_index }
}