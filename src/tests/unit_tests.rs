use crate::ast_parser::ast_parser::add_leaf;

#[test]
fn hello_world() {
    assert_eq!(2 + 2, 4);
}

// AST Parser Tests
#[test]
fn test_create_leaf() {
    let leaf_node = add_leaf("data".to_string());
    assert_eq!(leaf_node.data,"data");
}