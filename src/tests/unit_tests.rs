use crate::parser::{parser::{parse_query, parse_dist_query, is_comma, is_tab},ast::{Query, StructureElem}};

// AST Helper Functions
#[test]
fn test_is_comma() {
    assert!(is_comma(','));
}

#[test]
fn test_is_not_comma() {
    assert!(is_comma('h') == false);
}

#[test]
fn test_is_tab() {
    assert!(is_tab('\t'))
}

#[test]
fn test_is_not_tab() {
    assert!(is_tab(',') == false)
}

// AST Parser Tests
#[test]
fn test_freehand_query() {
    
    let (nxt, query) = parse_query("hello     world").unwrap();
    
    let tokens = match *query {
        Query::FreetextQuery {tokens} => tokens,
        _ => return assert_eq!(false,true,"Wrong type of query returned"),
    };

    assert_eq!(tokens[0],"hello");
    assert_eq!(tokens[1], "world");
}

#[test]
fn test_dist_query() {

    let query = "#DIST,3,pumpkin,pie";
    let (s, dist_node) = parse_dist_query(query).unwrap();
    match *dist_node {
        Query::DistanceQuery{dst, lhs, rhs} => assert!(dst == 3 && lhs == "pumpkin" && rhs == "pie"),
        _ => assert!(false)
    }

}

#[test]
fn test_dist_query_2() {

    let query = "#DIST 3 pumpkin pie";
    let (s, dist_node) = parse_dist_query(query).unwrap();
    match *dist_node {
        Query::DistanceQuery{dst, lhs, rhs} => assert!(dst == 3 && lhs == "pumpkin" && rhs == "pie"),
        _ => assert!(false)
    }

}