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


#[test]
fn test_category_query() {
    let (nxt, query) = parse_query("#CATEGORY hello").unwrap();
    
    let (elem,sub) = match *query {
        Query::StructureQuery {elem, sub} => (elem,sub),
        _ => return assert_eq!(false,true,"Wrong type of query returned"),
    };

    let tokens = match *sub {
        Query::FreetextQuery {tokens} => tokens,
        _ => return assert_eq!(false,true,"Wrong type of sub query returned"),
    };

    assert_eq!(elem,StructureElem::Category);
    assert_eq!(tokens[0], "hello");
}

#[test]
fn test_title_query() {
    let (nxt, query) = parse_query("#TITLE hello").unwrap();
    
    let (elem,sub) = match *query {
        Query::StructureQuery {elem, sub} => (elem,sub),
        _ => return assert_eq!(false,true,"Wrong type of query returned"),
    };

    let tokens = match *sub {
        Query::FreetextQuery {tokens} => tokens,
        _ => return assert_eq!(false,true,"Wrong type of sub query returned"),
    };

    assert_eq!(elem,StructureElem::Title);
    assert_eq!(tokens[0], "hello");
}

#[test]
fn test_citation_query() {
    let (nxt, query) = parse_query("#CITATION hello").unwrap();
    
    let (elem,sub) = match *query {
        Query::StructureQuery {elem, sub} => (elem,sub),
        _ => return assert_eq!(false,true,"Wrong type of query returned"),
    };

    let tokens = match *sub {
        Query::FreetextQuery {tokens} => tokens,
        _ => return assert_eq!(false,true,"Wrong type of sub query returned"),
    };

    assert_eq!(elem,StructureElem::Citation);
    assert_eq!(tokens[0], "hello");
}