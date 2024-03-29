use crate::{
    ast::{BinaryOp, Query, StructureElem, UnaryOp},
    parser::parse_query,
};

// AST Helper Functions

#[macro_export]
macro_rules! test_parse_and_print {
    ($name:ident, $target:expr ) => {
        #[test]
        fn $name() {
            let intermediate = format!("{}",$target);
            assert_eq!(parse_query(&intermediate).unwrap().1,$target, "Query (right) did not parse back into its original representation (left), print was: {}",&intermediate);
        }
    };
}

#[macro_export]
macro_rules! test_parse_to {
    ($name:ident,$string:expr, $target:expr ) => {
        #[test]
        fn $name() {
            assert_eq!(parse_query($string).unwrap().1,$target, "Query (right) was not parsed correctly from string (left), string was: {}",$string);
        }
    };
}




// print and parse back tests

test_parse_and_print!(test_parse_print_freetext, Box::new(Query::FreetextQuery { tokens: vec!["world".to_string(),"dog".to_string()] }));
test_parse_and_print!(test_parse_print_freetext_2, Box::new(Query::FreetextQuery { tokens: vec!["world".to_string()] }));
test_parse_and_print!(test_parse_print_phrase, Box::new(Query::PhraseQuery { tks: vec!["world".to_string(),"dog".to_string()] }));
test_parse_and_print!(test_parse_print_phrase_2, Box::new(Query::PhraseQuery { tks: vec!["world".to_string()] }));
test_parse_and_print!(test_parse_print_dist, Box::new(Query::DistanceQuery { dst: 3, lhs: "world".to_string(), rhs: "dog".to_string() }));
test_parse_and_print!(test_parse_print_structure, Box::new(Query::StructureQuery { 
    elem: StructureElem::Title, 
    sub:  Box::new(Query::FreetextQuery { tokens: vec!["world".to_string(),"dog".to_string()] })
}));

test_parse_and_print!(test_parse_print_structure_2, Box::new(Query::StructureQuery { 
    elem: StructureElem::Category, 
    sub:  Box::new(Query::FreetextQuery { tokens: vec!["world".to_string(),"dog".to_string()] })
}));

test_parse_and_print!(test_parse_print_structure_3, Box::new(Query::StructureQuery { 
    elem: StructureElem::Title, 
    sub:  Box::new(Query::FreetextQuery { tokens: vec!["world".to_string(),"dog".to_string()] })
}));

test_parse_and_print!(test_parse_print_structure_4, Box::new(Query::StructureQuery { 
    elem: StructureElem::Infobox("lmao".to_string()), 
    sub:  Box::new(Query::FreetextQuery { tokens: vec!["world".to_string(),"dog".to_string()] })
}));

test_parse_and_print!(test_parse_print_structure_5, Box::new(Query::StructureQuery { 
    elem: StructureElem::Infobox("dista".to_string()), 
    sub:  Box::new(Query::FreetextQuery { tokens: vec!["world".to_string(),"dog".to_string()] })
}));

test_parse_and_print!(test_parse_print_wildcard, Box::new(Query::WildcardQuery { 
    prefix: "".to_string(), 
    suffix: "w".to_string(), 
} ));

test_parse_and_print!(test_parse_print_wildcard_2, Box::new(Query::WildcardQuery { 
    prefix: "w".to_string(), 
    suffix: "".to_string(), 
} ));

test_parse_and_print!(test_parse_print_wildcard_3, Box::new(Query::WildcardQuery { 
    prefix: "h".to_string(), 
    suffix: "w".to_string(), 
} ));

test_parse_and_print!(test_parse_print_relational, Box::new(Query::RelationQuery { 
    root: 5, 
    hops: 2, 
    sub: Some(Box::new(Query::FreetextQuery { tokens: vec!["hello".to_string(),"dog".to_string()] })) 
} ));

test_parse_and_print!(test_parse_print_relational_2, Box::new(Query::RelationQuery { 
    root: 5, 
    hops: 2, 
    sub: None 
} ));

test_parse_and_print!(test_parse_print_not, Box::new(Query::UnaryQuery { 
    op: UnaryOp::Not, 
    sub: Box::new(Query::FreetextQuery { tokens: vec!["hello".to_string(),"dog".to_string()] }) 
} ));

test_parse_and_print!(test_parse_print_and, Box::new(Query::BinaryQuery { 
    op: BinaryOp::And,
    lhs: Box::new(Query::FreetextQuery { tokens: vec!["dog".to_string(),"world".to_string()] }),
    rhs: Box::new(Query::FreetextQuery { tokens: vec!["fish".to_string(),"hello".to_string()] }),
 }));

 test_parse_and_print!(test_parse_print_or, Box::new(Query::BinaryQuery { 
    op: BinaryOp::Or,
    lhs: Box::new(Query::FreetextQuery { tokens: vec!["dog".to_string(),"world".to_string()] }),
    rhs: Box::new(Query::FreetextQuery { tokens: vec!["fish".to_string(),"hello".to_string()] }),
 }));

 test_parse_and_print!(test_parse_print_complex_1, Box::new(Query::BinaryQuery { 
    op: BinaryOp::And,
    lhs: Box::new(Query::BinaryQuery { 
        op: BinaryOp::Or, 
        lhs: Box::new(Query::FreetextQuery { tokens: vec!["pORg".to_string(),"hello".to_string()] }), 
        rhs: Box::new(Query::FreetextQuery { tokens: vec!["fish".to_string(),"pORg".to_string()] }) 
    }),
    rhs: Box::new(Query::FreetextQuery { tokens: vec!["fish".to_string(),"hello".to_string()] }),
 }));


 test_parse_and_print!(test_parse_print_complex_2, Box::new(Query::BinaryQuery { 
    op: BinaryOp::And,
    lhs: Box::new(Query::BinaryQuery { 
        op: BinaryOp::Or, 
        lhs: Box::new(Query::FreetextQuery { tokens: vec!["fish".to_string(),"hello".to_string()] }), 
        rhs: Box::new(Query::FreetextQuery { tokens: vec!["fish".to_string(),"pORg".to_string()] }) 
    }),
    rhs:Box::new(Query::StructureQuery{ elem: StructureElem::Category, sub: Box::new(Query::FreetextQuery{tokens: vec!["FishORernios".to_string()]}) }),
 }));


 test_parse_to!(test_parse_complex_1, "#CATEGORY BOR AND #CATEGORY TOR",
    Box::new(Query::StructureQuery { 
        elem: StructureElem::Category, 
        sub: Box::new(Query::BinaryQuery { 
            op: BinaryOp::And, 
            lhs: Box::new(Query::FreetextQuery { tokens: vec!["BOR".to_string()] }), 
            rhs: Box::new(Query::StructureQuery { elem: StructureElem::Category, sub: Box::new(Query::FreetextQuery { tokens: vec!["TOR".to_string()] }) }), 
        }),
    })
);


test_parse_to!(test_parse_complex_2, "#DIST 3 BOR AND",
Box::new(Query::DistanceQuery { 
    dst: 3,
    lhs: "BOR".to_string(),
    rhs: "AND".to_string(),
}));


test_parse_to!(test_parse_complex_3, "#LINKSTO 1337 3 #DIST 3 BOR AND",
Box::new(Query::RelationQuery{ 
    root: 1337, 
    hops: 3, 
    sub: Some(Box::new(Query::DistanceQuery { dst: 3, lhs: "BOR".to_string(), rhs: "AND".to_string() })) 
}));

test_parse_to!(test_parse_complex_4, "#LINKSTO 1337 3 #CATEGORY #DIST 3 BOR AND",
Box::new(Query::RelationQuery{ 
    root: 1337, 
    hops: 3, 
    sub: Some(
        Box::new(Query::StructureQuery { 
            elem: StructureElem::Category, 
            sub: Box::new(Query::DistanceQuery { dst: 3, lhs: "BOR".to_string(), rhs: "AND".to_string() })
        })
    ) 
}));

test_parse_to!(test_parse_complex_5, "#LINKSTO 1337 3 NOT #CATEGORY  #DIST 3 BOR AND",
Box::new(Query::RelationQuery{ 
    root: 1337, 
    hops: 3, 
    sub: Some(
        Box::new(Query::UnaryQuery { 
            op: UnaryOp::Not, 
            sub:  
            Box::new(Query::StructureQuery { 
                elem: StructureElem::Category, 
                sub: Box::new(Query::DistanceQuery { dst: 3, lhs: "BOR".to_string(), rhs: "AND".to_string() })
            })
        })
       
    ) 
}));

test_parse_to!(test_parse_complex_6, "#LINKSTO 1337 3 #CATEGORY NOT #DIST 3 BOR AND",
Box::new(Query::RelationQuery{ 
    root: 1337, 
    hops: 3, 
    sub: Some(
        Box::new(Query::StructureQuery { 
            elem: StructureElem::Category, 
            sub: 
            Box::new(Query::UnaryQuery { 
                op: UnaryOp::Not, 
                sub: Box::new(Query::DistanceQuery { dst: 3, lhs: "BOR".to_string(), rhs: "AND".to_string() })
            }) 
        })
    ) 
}));

test_parse_to!(test_parse_complex_7, "#LINKSTO 1337 3 #CATEGORY NOT \"april may\" AND #DIST 3 BOR AND",
Box::new(Query::RelationQuery{ 
    root: 1337, 
    hops: 3, 
    sub: Some(
        Box::new(Query::StructureQuery { 
            elem: StructureElem::Category, 
            sub: 
            Box::new(Query::UnaryQuery { 
                op: UnaryOp::Not, 
                sub: Box::new(Query::BinaryQuery { 
                    op: BinaryOp::And, 
                    lhs: Box::new(Query::PhraseQuery { tks: vec!["april".to_string(),"may".to_string()] }), 
                    rhs: Box::new(Query::DistanceQuery { dst: 3, lhs: "BOR".to_string(), rhs: "AND".to_string() })
                })
            }) 
        })
    ) 
}));

test_parse_to!(test_parse_complex_8, "#distobox #LINKSTO 1337 3 NOT april*may AND #DIST 3 BOR AND",
Box::new(Query::StructureQuery{ 
    elem: StructureElem::Infobox("distobox".to_string()),
    sub: 
        Box::new(Query::RelationQuery { 
            root: 1337, 
            hops: 3, 
            sub: 
            Some(Box::new(Query::UnaryQuery { 
                op: UnaryOp::Not, 
                sub: Box::new(Query::BinaryQuery { 
                    op: BinaryOp::And, 
                    lhs: Box::new(Query::WildcardQuery { prefix: "april".to_string(), suffix: "may".to_string() }), 
                    rhs: Box::new(Query::DistanceQuery { dst: 3, lhs: "BOR".to_string(), rhs: "AND".to_string() })
                })
            }))
        })
    
}));

test_parse_to!(test_parse_complex_9, "a,AND,\"b\",AND,NOT,c,AND,d,AND,#CATEGORY,g,AND,#h i,AND,#DIST,1,e,f",
Box::new(Query::BinaryQuery { 
    op: BinaryOp::And,
    lhs: Box::new(Query::FreetextQuery { tokens: vec!["a".to_string()] }), 
    rhs: Box::new(Query::BinaryQuery { 
        op: BinaryOp::And, 
        lhs: Box::new(Query::PhraseQuery { tks: vec!["b".to_string()] }), 
        rhs: Box::new(Query::UnaryQuery { 
            op: UnaryOp::Not, 
            sub: Box::new(Query::BinaryQuery { 
                op: BinaryOp::And, 
                lhs: Box::new(Query::FreetextQuery { tokens: vec!["c".to_string()] }), 
                rhs: Box::new(Query::BinaryQuery { 
                    op: BinaryOp::And, 
                    lhs: Box::new(Query::FreetextQuery { tokens: vec!["d".to_string()] }), 
                    rhs: Box::new(Query::StructureQuery { 
                        elem: StructureElem::Category, 
                        sub: Box::new(Query::BinaryQuery { 
                            op: BinaryOp::And, 
                            lhs: Box::new(Query::FreetextQuery { tokens: vec!["g".to_string()] }), 
                            rhs: Box::new(Query::StructureQuery { 
                                elem: StructureElem::Infobox("h".to_string()), 
                                sub: Box::new(Query::BinaryQuery { 
                                    op: BinaryOp::And, 
                                    lhs: Box::new(Query::FreetextQuery { tokens: vec!["i".to_string()] }), 
                                    rhs: Box::new(Query::DistanceQuery { 
                                        dst: 1, 
                                        lhs: "e".to_string(), 
                                        rhs: "f".to_string()
                                    })
                                }) 
                            })
                        }) 
                    }) 
                })
            }) 
        }), 
    }),
    })

);

test_parse_to!(test_parse_complex_10, "a,AND,\"b\",AND,d,AND,#CATEGORY,g,AND,#h i,AND,#DIST,1,e,f, AND NOT c",
Box::new(Query::BinaryQuery { 
    op: BinaryOp::And,
    lhs: Box::new(Query::BinaryQuery{
        op: BinaryOp::And,
        lhs: Box::new(Query::FreetextQuery { tokens: vec!["a".to_string()] }), 
        rhs: Box::new(Query::BinaryQuery { 
            op: BinaryOp::And, 
            lhs: Box::new(Query::PhraseQuery { tks: vec!["b".to_string()] }), 
            rhs:Box::new(Query::BinaryQuery { 
                    op: BinaryOp::And, 
                    lhs: Box::new(Query::FreetextQuery { tokens: vec!["d".to_string()] }), 
                    rhs: Box::new(Query::StructureQuery{ 
                        elem: StructureElem::Category, 
                        sub: Box::new(Query::BinaryQuery { 
                            op: BinaryOp::And, 
                            lhs: Box::new(Query::FreetextQuery { tokens: vec!["g".to_string()] }), 
                            rhs: Box::new(Query::StructureQuery { 
                                elem: StructureElem::Infobox("h".to_string()), 
                                sub: Box::new(Query::BinaryQuery { 
                                    op: BinaryOp::And, 
                                    lhs: Box::new(Query::FreetextQuery { tokens: vec!["i".to_string()] }), 
                                    rhs: Box::new(Query::DistanceQuery { dst: 1, lhs: "e".to_string(), rhs: "f".to_string() }) 
                                }) 
                            })
                        }) 
                    })
                })
        })
    }),
    rhs: Box::new(Query::UnaryQuery { 
        op: UnaryOp::Not, 
        sub: Box::new(Query::FreetextQuery { tokens: vec!["c".to_string()] })
    })  
}));

// AST Parser Tests
#[test]
fn test_freehand_query() {
    let (_nxt, query) = parse_query(" hello     world ").unwrap();

    let tokens = match *query {
        Query::FreetextQuery { tokens } => tokens,
        _ => return assert_eq!(false, true, "Wrong type of query returned"),
    };

    assert_eq!(tokens[0], "hello");
    assert_eq!(tokens[1], "world");
}

#[test]
fn test_dist_query() {
    let query = " #DIST , 3 , pumpkin,pie ";
    let (_s, dist_node) = parse_query(query).unwrap();
    match *dist_node {
        Query::DistanceQuery { dst, lhs, rhs } => {
            assert!(dst == 3 && lhs == "pumpkin" && rhs == "pie")
        }
        _ => assert!(false),
    }
}

#[test]
fn test_dist_query_2() {
    let query = " #DIST 3 pumpkin pie ";
    let (_s, dist_node) = parse_query(query).unwrap();
    match *dist_node {
        Query::DistanceQuery { dst, lhs, rhs } => {
            assert!(dst == 3 && lhs == "pumpkin" && rhs == "pie")
        }
        _ => assert!(false),
    }
}

#[test]
fn test_complex_dist_with_binary_query() {
    let query = "borisss, AND, #DIST,4,boris ,johnson,";

    let (_s, dist_node) = parse_query(query).unwrap();

    let l = Box::new(Query::FreetextQuery {
        tokens: vec!["borisss".to_string()],
    });
    let r = Box::new(Query::DistanceQuery {
        dst: 4,
        lhs: "boris".to_string(),
        rhs: "johnson".to_string(),
    });
    let expected = Box::new(
        Query::BinaryQuery { op: (BinaryOp::And), lhs: (l), rhs: (r) }
    );
    
    assert_eq!(expected, dist_node);
}

# [test]
fn test_complex_dist_with_binary_query_2() {
    let query = "borisss, AND, #DIST,4,boris ,johnson,OR,#CITATION,boris";

    let (_s, dist_node) = parse_query(query).unwrap();

    let l = Box::new(Query::FreetextQuery {
        tokens: vec!["borisss".to_string()],
    });
    let r1 = Box::new(Query::DistanceQuery {
        dst: 4,
        lhs: "boris".to_string(),
        rhs: "johnson".to_string(),
    });
    let r2 = Box::new(Query::StructureQuery { 
        elem: (StructureElem::Citation), 
        sub: (Box::new(
            Query::FreetextQuery { 
                tokens: (vec!["boris".to_string()]) 
            })) 
    });
    let expected = Box::new(
        Query::BinaryQuery {
             op: (BinaryOp::And), 
             lhs: (l), 
             rhs: (Box::new(
                 Query::BinaryQuery { 
                     op: (BinaryOp::Or), 
                     lhs: (r1), 
                     rhs: (r2) 
                    }
                )
                
            ) 
        }
    );
    
    assert_eq!(expected, dist_node);
}

#[test]
fn test_simple_structure_query() {
    let query = " #TITLE pumpkin ";
    let (_s, struct_node) = parse_query(query).unwrap();
    match *struct_node {
        Query::StructureQuery { elem, sub } => assert!(
            elem == StructureElem::Title
                && sub
                    == Box::new(Query::FreetextQuery {
                        tokens: vec!["pumpkin".to_string()],
                    })
        ),
        _ => assert!(false),
    }
}

#[test]
fn test_simple_not_query() {
    let query = "NOT pumpkin";
    let (_s, unary_node) = parse_query(query).unwrap();
    match *unary_node {
        Query::UnaryQuery { op, sub } => assert!(
            op == UnaryOp::Not
                && sub
                    == Box::new(Query::FreetextQuery {
                        tokens: vec!["pumpkin".to_string()],
                    })
        ),
        _ => assert!(false),
    }
}



test_parse_to!(test_simple_or_query,
    "pumpkin OR pie",
    Box::new(Query::BinaryQuery { 
        op: BinaryOp::Or, 
        lhs: Box::new(Query::FreetextQuery { tokens: vec!["pumpkin".to_string()] }), 
        rhs: Box::new(Query::FreetextQuery { tokens: vec!["pie".to_string()] }) 
}));

#[test]
fn test_phrase_freetext_query () {
    let query = "fresh,AND,\"goat\"";

    let (_, actual) = parse_query(query).unwrap();
    let lhs = Box::new(Query::FreetextQuery { tokens: (vec!["fresh".to_string()]) });
    let rhs = Box::new(Query::PhraseQuery { tks: (vec!["goat".to_string()]) });

    let expected = Box::new(Query::BinaryQuery { op: (BinaryOp::And), lhs: (lhs), rhs: (rhs) });

    assert_eq!(expected, actual);
}

#[test]
fn test_multitoken_or_query() {
    let query = "pumpkin pie OR pumpkin patch";
    let l = Box::new(Query::FreetextQuery {
        tokens: vec!["pumpkin".to_string(), "pie".to_string()],
    });
    let r = Box::new(Query::FreetextQuery {
        tokens: vec!["pumpkin".to_string(), "patch".to_string()],
    });
    let (_s, binary_node) = parse_query(query).unwrap();
    match *binary_node {
        Query::BinaryQuery { op, lhs, rhs } => assert!(op == BinaryOp::Or && lhs == l && rhs == r),
        _ => assert!(false),
    }
}

#[test]
fn test_simple_and_query() {
    let query = "pumpkin AND pie";
    let l = Box::new(Query::FreetextQuery {
        tokens: vec!["pumpkin".to_string()],
    });
    let r = Box::new(Query::FreetextQuery {
        tokens: vec!["pie".to_string()],
    });
    let (_s, binary_node) = parse_query(query).unwrap();
    match *binary_node {
        Query::BinaryQuery { op, lhs, rhs } => assert!(op == BinaryOp::And && lhs == l && rhs == r),
        _ => assert!(false),
    }
}

#[test]
fn test_multitoken_and_query() {
    let query = "pumpkin pie AND pumpkin patch";
    let l = Box::new(Query::FreetextQuery {
        tokens: vec!["pumpkin".to_string(), "pie".to_string()],
    });
    let r = Box::new(Query::FreetextQuery {
        tokens: vec!["pumpkin".to_string(), "patch".to_string()],
    });
    let (_s, binary_node) = parse_query(query).unwrap();
    match *binary_node {
        Query::BinaryQuery { op, lhs, rhs } => assert!(op == BinaryOp::And && lhs == l && rhs == r),
        _ => assert!(false),
    }
}

#[test]
fn test_simple_binary_query() {
    let query = "pumpkin pie AND pumpkin patch";
    let l = Box::new(Query::FreetextQuery {
        tokens: vec!["pumpkin".to_string(), "pie".to_string()],
    });
    let r = Box::new(Query::FreetextQuery {
        tokens: vec!["pumpkin".to_string(), "patch".to_string()],
    });
    let (_s, binary_node) = parse_query(query).unwrap();
    match *binary_node {
        Query::BinaryQuery { op, lhs, rhs } => assert!(op == BinaryOp::And && lhs == l && rhs == r),
        _ => assert!(false),
    }
}

#[test]
fn test_simple_binary_query_2() {
    let query = "pumpkin pie OR pumpkin patch";
    let l = Box::new(Query::FreetextQuery {
        tokens: vec!["pumpkin".to_string(), "pie".to_string()],
    });
    let r = Box::new(Query::FreetextQuery {
        tokens: vec!["pumpkin".to_string(), "patch".to_string()],
    });
    let (_s, binary_node) = parse_query(query).unwrap();
    match *binary_node {
        Query::BinaryQuery { op, lhs, rhs } => assert!(op == BinaryOp::Or && lhs == l && rhs == r),
        _ => assert!(false),
    }
}

#[test]
fn test_nested_binary_query() {
    let query = "pumpkin pie AND pumpkin patch AND pumpkin spice latte";
    let l = Box::new(Query::FreetextQuery {
        tokens: vec!["pumpkin".to_string(), "pie".to_string()],
    });
    let l2 = Box::new(Query::FreetextQuery {
        tokens: vec!["pumpkin".to_string(), "patch".to_string()],
    });
    let r2 = Box::new(Query::FreetextQuery {
        tokens: vec![
            "pumpkin".to_string(),
            "spice".to_string(),
            "latte".to_string(),
        ],
    });
    let r = Box::new(Query::BinaryQuery {
        op: BinaryOp::And,
        lhs: l2,
        rhs: r2,
    });

    let (_s, binary_node) = parse_query(query).unwrap();
    match *binary_node {
        Query::BinaryQuery { op, lhs, rhs } => assert!(op == BinaryOp::And && lhs == l && rhs == r),
        _ => assert!(false),
    }
}

#[test]
fn test_parse_simple_wildcard_query() {
    let query = "p*kin";
    let expected = Query::WildcardQuery {
        prefix: "p".to_string(),
        suffix: "kin".to_string(),
    };
    let (_s, wildcard_query) = parse_query(query).unwrap();
    match *wildcard_query {
        q => assert!(q == expected),
    }
}

#[test]
fn test_parse_wildcard_query_with_whitespace() {
    let query = " p * kin           ";
    let expected = Query::WildcardQuery {
        prefix: "p".to_string(),
        suffix: "kin".to_string(),
    };
    let (_s, wildcard_query) = parse_query(query).unwrap();
    match *wildcard_query {
        q => assert!(q == expected),
    }
}

#[test]
fn test_parse_wildcard_query_no_prefix() {
    let query = "*kin";
    let expected = Query::WildcardQuery {
        prefix: "".to_string(),
        suffix: "kin".to_string(),
    };
    let (_s, wildcard_query) = parse_query(query).unwrap();
    match *wildcard_query {
        q => assert!(q == expected),
    }
}

#[test]
fn test_parse_simple_wildcard_query_no_suffix() {
    let query = "p*";
    let expected = Query::WildcardQuery {
        prefix: "p".to_string(),
        suffix: "".to_string(),
    };
    let (_s, wildcard_query) = parse_query(query).unwrap();
    match *wildcard_query {
        q => assert!(q == expected),
    }
}

#[test]
fn test_wildcard_query() {
    let query = "a*ril";
    let expected = Box::new(Query::WildcardQuery {
        prefix: "a".to_string(),
        suffix: "ril".to_string(),
    });
    assert_eq!(parse_query(query), Ok(("", expected)));
}

#[test]
fn test_wildcard_query_2() {
    let query = "alche*";
    let expected = Box::new(Query::WildcardQuery {
        prefix: "alche".to_string(),
        suffix: "".to_string(),
    });
    assert_eq!(parse_query(query), Ok(("", expected)));
}

#[test]
fn test_binary_with_wildcard_query() {
    let query = "pumpk*n AND pie";
    let l = Box::new(Query::WildcardQuery {
        prefix: "pumpk".to_string(),
        suffix: "n".to_string(),
    });
    let r = Box::new(Query::FreetextQuery {
        tokens: vec!["pie".to_string()],
    });
    let (_s, binary_node) = parse_query(query).unwrap();

    let target = Box::new(Query::BinaryQuery {
        op: BinaryOp::And,
        lhs: l,
        rhs: r,
    });
    assert_eq!(target, binary_node);
}

#[test]
fn test_compound_query_or_and_with_wildcard() {
    let query = "pumpkin pie AND pumpkin OR p*tch";

    assert_eq!(
        parse_query(query),
        Ok((
            "",
            Box::new(Query::BinaryQuery {
                lhs: Box::new(Query::FreetextQuery {
                    tokens: vec!["pumpkin".to_string(), "pie".to_string()],
                }),
                op: BinaryOp::And,
                rhs: Box::new(Query::BinaryQuery {
                    lhs: Box::new(Query::FreetextQuery {
                        tokens: vec!["pumpkin".to_string()],
                    }),
                    op: BinaryOp::Or,
                    rhs: Box::new(Query::WildcardQuery {
                        prefix: "p".to_string(),
                        suffix: "tch".to_string()
                    }),
                }),
            })
        ))
    );
}

#[test]
fn test_not_with_wildcard() {
    let query = "NOT ca*";
    let (_s, unary_node) = parse_query(query).unwrap();
    let target = Box::new(Query::UnaryQuery {
        op: UnaryOp::Not,
        sub: Box::new(Query::WildcardQuery {
            prefix: "ca".to_string(),
            suffix: "".to_string(),
        }),
    });

    assert_eq!(target, unary_node)
}

#[test]
fn test_compound_query_or_and() {
    let query = "pumpkin pie OR pumpkin AND patch";

    assert_eq!(
        parse_query(query),
        Ok((
            "",
            Box::new(Query::BinaryQuery {
                lhs: Box::new(Query::BinaryQuery {
                    lhs: Box::new(Query::FreetextQuery {
                        tokens: vec!["pumpkin".to_string(), "pie".to_string()],
                    }),
                    op: BinaryOp::Or,
                    rhs: Box::new(Query::FreetextQuery {
                        tokens: vec!["pumpkin".to_string()]
                    }),
                }),
                op: BinaryOp::And,
                rhs: Box::new(Query::FreetextQuery {
                    tokens: vec!["patch".to_string()],
                }),
            })
        ))
    );
}

#[test]
fn test_compound_query_or_and_2() {
    let query = "pumpkin pie AND pumpkin OR patch";

    assert_eq!(
        parse_query(query),
        Ok((
            "",
            Box::new(Query::BinaryQuery {
                lhs: Box::new(Query::FreetextQuery {
                    tokens: vec!["pumpkin".to_string(), "pie".to_string()],
                }),
                op: BinaryOp::And,
                rhs: Box::new(Query::BinaryQuery {
                    lhs: Box::new(Query::FreetextQuery {
                        tokens: vec!["pumpkin".to_string()],
                    }),
                    op: BinaryOp::Or,
                    rhs: Box::new(Query::FreetextQuery {
                        tokens: vec!["patch".to_string()]
                    }),
                }),
            })
        ))
    );
}

#[test]
fn test_parse_simple_phrase_query_no_spaces() {
    let query = "\"April\"";
    let expected_tks = vec!["April".to_string()];
    let expected = Box::new(Query::PhraseQuery { tks: expected_tks });
    let (_, actual) = parse_query(query).unwrap();
    assert!(actual == expected);
}

#[test]
fn test_parse_simple_phrase_query() {
    let query = " \" The big whale ate a tuna sandwich. \" ";
    let expected_tks = vec![
        "The".to_string(),
        "big".to_string(),
        "whale".to_string(),
        "ate".to_string(),
        "a".to_string(),
        "tuna".to_string(),
        "sandwich".to_string(),
    ];
    let expected = Box::new(Query::PhraseQuery { tks: expected_tks });
    let (_, actual) = parse_query(query).unwrap();
    assert!(actual == expected);
}

#[test]
fn test_parse_complex_phrase_query() {
    let query = " \" The  , big ,,,, whale  , ate              a ,tuna     sandwich. \" ";
    let expected_tks = vec![
        "The".to_string(),
        "big".to_string(),
        "whale".to_string(),
        "ate".to_string(),
        "a".to_string(),
        "tuna".to_string(),
        "sandwich".to_string(),
    ];
    let expected = Box::new(Query::PhraseQuery { tks: expected_tks });
    let (_, actual) = parse_query(query).unwrap();
    assert!(actual == expected);
}

#[test]
fn test_parse_simple_relation_query() {
    let query = " #LINKSTO, 173302 ,1 ";
    let expected_root = 173302;
    let expected_hops = 1;
    let expected_sub = None;
    let expected = Box::new(Query::RelationQuery {
        root: expected_root,
        hops: expected_hops,
        sub: expected_sub,
    });
    let (_, actual) = parse_query(query).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_parse_simple_relation_query2() {
    let query = " #LINKSTO  69  3 ";
    let expected_root = 69;
    let expected_hops = 3;
    let expected_sub = None;
    let expected = Box::new(Query::RelationQuery {
        root: expected_root,
        hops: expected_hops,
        sub: expected_sub,
    });
    let (_, actual) = parse_query(query).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_parse_simple_relation_query3() {
    let query = "#LINKSTO, 142, 255";
    let expected_root = 142;
    let expected_hops = 255;
    let expected_sub = None;
    let expected = Box::new(Query::RelationQuery {
        root: expected_root,
        hops: expected_hops,
        sub: expected_sub,
    });
    let (_, actual) = parse_query(query).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_parse_nested_relation_query() {
    let query = "#LINKSTO , 2222 , 0 , Donald OR Trump  ";
    let expected_root = 2222;
    let expected_hops = 0;
    let expected_sub = Box::new(Query::BinaryQuery {
        op: BinaryOp::Or,
        lhs: Box::new(Query::FreetextQuery {
            tokens: vec!["Donald".to_string()],
        }),
        rhs: Box::new(Query::FreetextQuery {
            tokens: vec!["Trump".to_string()],
        }),
    });
    let expected = Box::new(Query::RelationQuery {
        root: expected_root,
        hops: expected_hops,
        sub: Some(expected_sub),
    });
    let (_, actual) = parse_query(query).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_parse_query_with_structure_query() {
    let query = "#TITLE April";
    let expected = Box::new(Query::StructureQuery {
        elem: StructureElem::Title,
        sub: Box::new(Query::FreetextQuery {
            tokens: vec!["April".to_string()],
        }),
    });

    let (_, actual) = parse_query(query).unwrap();

    assert_eq!(expected, actual);
}

#[test]
fn test_complex_structural_binary_query() {
    let query = "Boris,AND,Johnson,AND,#CATEGORY, Prime Ministers of the United Kingdom";
    let expected = Box::new(Query::BinaryQuery {
        op: BinaryOp::And,
        lhs: Box::new(Query::FreetextQuery {
            tokens: vec!["Boris".to_string()],
        }),
        rhs: Box::new(Query::BinaryQuery {
            op: BinaryOp::And,
            lhs: Box::new(Query::FreetextQuery {
                tokens: vec!["Johnson".to_string()],
            }),
            rhs: Box::new(Query::StructureQuery {
                elem: StructureElem::Category,
                sub: Box::new(Query::FreetextQuery {
                    tokens: vec![
                        "Prime".to_string(),
                        "Ministers".to_string(),
                        "of".to_string(),
                        "the".to_string(),
                        "United".to_string(),
                        "Kingdom".to_string(),
                    ],
                }),
            }),
        }),
    });
    let (_, actual) = parse_query(query).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_complex_structural_binary_query_2() {
    let query = "#CATEGORY, Prime Ministers of the United Kingdom,AND,Boris,AND,Johnson";
    let expected = Box::new(Query::StructureQuery {
        elem: StructureElem::Category,
        sub: Box::new(Query::BinaryQuery {
            op: BinaryOp::And,
            lhs: Box::new(Query::FreetextQuery {
                tokens: vec![
                    "Prime".to_string(),
                    "Ministers".to_string(),
                    "of".to_string(),
                    "the".to_string(),
                    "United".to_string(),
                    "Kingdom".to_string(),
                ],
            }),
            rhs: Box::new(Query::BinaryQuery {
                op: BinaryOp::And,
                lhs: Box::new(Query::FreetextQuery {
                    tokens: vec!["Boris".to_string()],
                }),
                rhs: Box::new(Query::FreetextQuery {
                    tokens: vec!["Johnson".to_string()],
                }),
            }),
        }),
    });
    let (_, actual) = parse_query(query).unwrap();
    assert_eq!(expected, actual);
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
