use crate::parser::{
    parser::{
        parse_query, 
        parse_dist_query, 
        is_comma, 
        is_tab, 
        parse_structure_query, 
        parse_not_query, 
        parse_or_query, 
        parse_and_query, 
        parse_binary_query,
        parse_wildcard_query,
    },
    ast::{Query, StructureElem, BinaryOp, UnaryOp}};

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

#[test]
fn test_simple_structure_query() {
    let query = "#TITLE pumpkin";
    let (s, struct_node) = parse_structure_query(query).unwrap();
    match *struct_node {
        Query::StructureQuery{elem, sub} => assert!(elem == StructureElem::Title && sub == Box::new(Query::FreetextQuery{
            tokens: vec!["pumpkin".to_string()],
        })),
        _ => assert!(false),
    }
}

#[test]
fn test_simple_not_query() {
    let query = "NOT pumpkin";
    let (s, unary_node) = parse_not_query(query).unwrap();
    match *unary_node {
        Query::UnaryQuery{op, sub} => assert!(op == UnaryOp::Not && sub == Box::new(Query::FreetextQuery{
            tokens: vec!["pumpkin".to_string()],
        })),
        _ => assert!(false),
    }
}

#[test]
fn test_simple_or_query() {
    let query = "pumpkin OR pie";
    let l = Box::new(Query::FreetextQuery{
        tokens: vec!["pumpkin".to_string()],
    });
    let r = Box::new(Query::FreetextQuery{
        tokens: vec!["pie".to_string()],
    });
    let (s, binary_node) = parse_or_query(query).unwrap();
    match *binary_node {
        Query::BinaryQuery{op, lhs, rhs} => assert!(op == BinaryOp::Or && lhs == l && rhs == r),
        _ => assert!(false),
    }

}

#[test]
fn test_multitoken_or_query() {
    let query = "pumpkin pie OR pumpkin patch";
    let l = Box::new(Query::FreetextQuery{
        tokens: vec!["pumpkin".to_string(), "pie".to_string()],
    });
    let r = Box::new(Query::FreetextQuery{
        tokens: vec!["pumpkin".to_string(), "patch".to_string()],
    });
    let (s, binary_node) = parse_or_query(query).unwrap();
    match *binary_node {
        Query::BinaryQuery{op, lhs, rhs} => assert!(op == BinaryOp::Or && lhs == l && rhs == r),
        _ => assert!(false),
    }

}

#[test]
fn test_simple_and_query() {
    let query = "pumpkin AND pie";
    let l = Box::new(Query::FreetextQuery{
        tokens: vec!["pumpkin".to_string()],
    });
    let r = Box::new(Query::FreetextQuery{
        tokens: vec!["pie".to_string()],
    });
    let (s, binary_node) = parse_and_query(query).unwrap();
    match *binary_node {
        Query::BinaryQuery{op, lhs, rhs} => assert!(op == BinaryOp::And && lhs == l && rhs == r),
        _ => assert!(false),
    }

}

#[test]
fn test_multitoken_and_query() {
    let query = "pumpkin pie AND pumpkin patch";
    let l = Box::new(Query::FreetextQuery{
        tokens: vec!["pumpkin".to_string(), "pie".to_string()],
    });
    let r = Box::new(Query::FreetextQuery{
        tokens: vec!["pumpkin".to_string(), "patch".to_string()],
    });
    let (s, binary_node) = parse_and_query(query).unwrap();
    match *binary_node {
        Query::BinaryQuery{op, lhs, rhs} => assert!(op == BinaryOp::And && lhs == l && rhs == r),
        _ => assert!(false),
    }

}

#[test]
fn test_simple_binary_query() {
    let query = "pumpkin pie AND pumpkin patch";
    let l = Box::new(Query::FreetextQuery{
        tokens: vec!["pumpkin".to_string(), "pie".to_string()],
    });
    let r = Box::new(Query::FreetextQuery{
        tokens: vec!["pumpkin".to_string(), "patch".to_string()],
    });
    let (s, binary_node) = parse_binary_query(query).unwrap();
    match *binary_node {
        Query::BinaryQuery{op, lhs, rhs} => assert!(op == BinaryOp::And && lhs == l && rhs == r),
        _ => assert!(false),
    }
}

#[test]
fn test_simple_binary_query_2() {
    let query = "pumpkin pie OR pumpkin patch";
    let l = Box::new(Query::FreetextQuery{
        tokens: vec!["pumpkin".to_string(), "pie".to_string()],
    });
    let r = Box::new(Query::FreetextQuery{
        tokens: vec!["pumpkin".to_string(), "patch".to_string()],
    });
    let (s, binary_node) = parse_binary_query(query).unwrap();
    match *binary_node {
        Query::BinaryQuery{op, lhs, rhs} => assert!(op == BinaryOp::Or && lhs == l && rhs == r),
        _ => assert!(false),
    }
}

#[test]
fn test_nested_binary_query() {
    let query = "pumpkin pie AND pumpkin patch AND pumpkin spice latte";
    let l = Box::new(Query::FreetextQuery{
        tokens: vec!["pumpkin".to_string(), "pie".to_string()],
    });
    let l2 = Box::new(Query::FreetextQuery{
        tokens: vec!["pumpkin".to_string(), "patch".to_string()],
    });
    let r2 = Box::new(Query::FreetextQuery{
        tokens: vec!["pumpkin".to_string(), "spice".to_string(), "latte".to_string()],
    });
    let r = Box::new(Query::BinaryQuery{
        op : BinaryOp::And,
        lhs: l2,
        rhs: r2
    });

    let (s, binary_node) = parse_binary_query(query).unwrap();
    match *binary_node {
        Query::BinaryQuery{op, lhs, rhs} => assert!(op == BinaryOp::And && lhs == l && rhs == r),
        _ => assert!(false),
    }
}

#[test]
fn test_parse_simple_wildcard_query() {
    let query = "p*kin";
    let expected = Query::WildcardQuery{
        prefix: "p".to_string(),
        postfix: "kin".to_string(),
    };
    let (s, wildcard_query) = parse_wildcard_query(query).unwrap();
    match *wildcard_query {
        q => assert!(q == expected),
        _ => assert!(false)
    }
}

#[test]
fn test_parse_wildcard_query_with_whitespace() {
    let query = " p * kin           ";
    let expected = Query::WildcardQuery{
        prefix: "p".to_string(),
        postfix: "kin".to_string(),
    };
    let (s, wildcard_query) = parse_wildcard_query(query).unwrap();
    match *wildcard_query {
        q => assert!(q == expected),
        _ => assert!(false)
    }
}

#[test]
fn test_parse_wildcard_query_no_prefix() {
    let query = "*kin";
    let expected = Query::WildcardQuery{
        prefix: "".to_string(),
        postfix: "kin".to_string(),
    };
    let (s, wildcard_query) = parse_wildcard_query(query).unwrap();
    match *wildcard_query {
        q => assert!(q == expected),
        _ => assert!(false)
    }
}

#[test]
fn test_parse_simple_wildcard_query_no_suffix() {
    let query = "p*";
    let expected = Query::WildcardQuery{
        prefix: "p".to_string(),
        postfix: "".to_string(),
    };
    let (s, wildcard_query) = parse_wildcard_query(query).unwrap();
    match *wildcard_query {
        q => assert!(q == expected),
        _ => assert!(false)
    }
}

#[test]
fn test_compound_query_or_and() {
    let query = "pumpkin pie OR pumpkin AND patch";


    assert_eq!(parse_query(query),Ok(("",
    Box::new(
        Query::BinaryQuery{
            lhs: Box::new(
                Query::BinaryQuery{
                    lhs: Box::new(
                        Query::FreetextQuery{
                            tokens: vec!["pumpkin".to_string(),"pie".to_string()],
                    }),
                    op:BinaryOp::Or,
                    rhs: Box::new(
                        Query::FreetextQuery{
                            tokens: vec!["pumpkin".to_string()]
                        }
                    ),
                }
            ),
            op: BinaryOp::And,
            rhs: Box::new(
                Query::FreetextQuery{
                    tokens: vec!["patch".to_string()],
                }
            ),
        }
    ))));
}

#[test]
fn test_compound_query_or_and_2() {
    let query = "pumpkin pie AND pumpkin OR patch";

    assert_eq!(parse_query(query),Ok(("",
    Box::new(
        Query::BinaryQuery{
            lhs: Box::new(
                Query::FreetextQuery{
                    tokens: vec!["pumpkin".to_string(),"pie".to_string()],
                }
            ),
            op: BinaryOp::And,
            rhs: Box::new(
                Query::BinaryQuery{
                    lhs: Box::new(
                        Query::FreetextQuery{
                            tokens: vec!["pumpkin".to_string()],
                    }),
                    op:BinaryOp::Or,
                    rhs: Box::new(
                        Query::FreetextQuery{
                            tokens: vec!["patch".to_string()]
                        }
                    ),
                }
            ),
        }
    ))));
}


// test for left associativity for both operators
// right associativity is fine too you can adapt this, but specify in the grammar the associativity and precedence
//
// #[test]
// fn test_compound_query_or_and_3() {
//     let query = "pumpkin AND pie AND pumpkin OR patch OR pie";

//     assert_eq!(parse_query(query),Ok(("",
//     Box::new(
//         Query::BinaryQuery{
//             lhs: Box::new(
//                 Query::BinaryQuery{
//                     lhs: Box::new(
//                         Query::FreetextQuery{
//                             tokens: vec!["pumpkin".to_string()]
//                         }
//                     ),
//                     op: BinaryOp::And,
//                     rhs: Box::new(
//                         Query::FreetextQuery{
//                             tokens: vec!["pie".to_string()]
//                         }
//                     ),
//                 }
//             ),
//             op: BinaryOp::And,
//             rhs: Box::new(
//                 Query::BinaryQuery{
//                     lhs: Box::new(
//                         Query::BinaryQuery{
//                             lhs: Box::new(
//                                 Query::FreetextQuery{
//                                     tokens: vec!["pumpkin".to_string()]
//                                 }
//                             ),
//                             op: BinaryOp::Or,
//                             rhs: Box::new(
//                                     Query::FreetextQuery{
//                                         tokens: vec!["patch".to_string()]
//                             }),
//                         }
//                     ),
//                     op:BinaryOp::Or,
//                     rhs: Box::new(
//                         Query::FreetextQuery{
//                             tokens: vec!["pie".to_string()]
//                         }
//                     ),
//                 }
//             ),
//         }
//     ))))
// }

