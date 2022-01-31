// Tree implementation, taken from https://rust-leipzig.github.io/architecture/2016/12/20/idiomatic-trees-in-rust/

pub struct Arena<T> {
    nodes: Vec<Node<T>>,
}

pub struct Node<T> {
    parent: Option<NodeId>,
    previous_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
    first_child: Option<NodeId>,
    last_child: Option<NodeId>,

    /// The actual data which will be stored within the tree
    pub data: T,
}

pub struct NodeId {
    index: usize,
}

pub fn new_node(&mut self, data: T) -> NodeId {
    // Get the next free index
    let next_index = self.nodes.len();

    // Push the node into the arena
    self.nodes.push(Node {
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