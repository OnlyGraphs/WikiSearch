use crate::ast_parser::ast_parser::add_leaf;
use crate::ast_parser::ast_parser::parse_term;
use crate::ast_parser::ast_parser::DistNode;
use crate::ast_parser::ast_parser::Node;
use nom::{
    IResult
};

// AST Parser Tests
#[test]
fn test_create_leaf() {
    let leaf_node = add_leaf("data".to_string());
    assert_eq!(leaf_node.data,"data");
}


#[test]
fn test_parse_dist_node(){
    let (nxt,node) = parse_dist_query("#DIST,2,asd,basd");
    let expected = DistNode {
        parent: None,
        previous_sibling: None,
        next_sibling: None,
        first_child: Node {
            parent: None,
            previous_sibling: None,
            next_sibling: None,
            data: "asd".to_string(),
        },
        last_child: Node {
            parent: None,
            previous_sibling: None,
            next_sibling: None,
            data: "basd".to_string(),
        },
        dist: 2,
    };

    assert_eq!(node, expected)
}

#[test]
fn test_parse_term() {
    let (x, y) = parse_term("henlo :)").unwrap();
    assert_eq!(x, " :)");
    assert_eq!(y.data, "henlo")
}

#[test]
fn test_parse_term_empty() {
    let (x, y) = parse_term("(: henlo :)").unwrap();
    assert_eq!(x, "(: henlo :)");
    assert_eq!(y.data, "")
}