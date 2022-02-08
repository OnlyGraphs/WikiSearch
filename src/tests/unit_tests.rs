use crate::parser::{parser::parse_query,ast::{Query, StructureElem}};


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