
use crate::parser::ast::{Query, BinaryOp, UnaryOp, StructureElem};
use crate::search::search::{preprocess_query};

#[test]
fn test_single_word(){

    let mut q = Query::FreetextQuery{
        tokens: vec!["BarCeLOna".to_string()]
    };

    preprocess_query(&mut q).unwrap();

    assert_eq!(q,
        Query::FreetextQuery{
            tokens: vec!["barcelona".to_string()]
        }
    )
}


#[test]
fn test_multiple_words(){

    let mut q = Query::FreetextQuery{
        tokens: vec!["BarCeLOna snickers".to_string()]
    };

    preprocess_query(&mut q).unwrap();

    assert_eq!(q,
        Query::FreetextQuery{
            tokens: vec!["barcelona".to_string(), "snicker".to_string()]
        }
    )
}

#[test]
fn test_stop_word(){

    let mut q = Query::FreetextQuery{
        tokens: vec!["the".to_string()]
    };

    preprocess_query(&mut q).unwrap();

    assert_eq!(q,
        Query::FreetextQuery{
            tokens: Vec::default()
        }
    )
}


#[test]
fn test_binary_query(){

    let mut q = 
     
    Query::BinaryQuery{
        lhs: Box::new(Query::FreetextQuery{
            tokens: vec!["the".to_string()]
        }),
        op: BinaryOp::And,
        rhs: Box::new(Query::FreetextQuery{
            tokens: vec!["bars".to_string()]
        }),
    };

    preprocess_query(&mut q).unwrap();

    assert_eq!(q,
        Query::BinaryQuery{
            lhs: Box::new(Query::FreetextQuery{
                tokens: Vec::default()
            }),
            op: BinaryOp::And,
            rhs: Box::new(Query::FreetextQuery{
                tokens: vec!["bar".to_string()]
            }),
        }
    )
}

#[test]
fn test_unary_query(){

    let mut q = 
     
    Query::UnaryQuery{
        sub: Box::new(Query::FreetextQuery{
            tokens: vec!["the".to_string()]
        }),
        op: UnaryOp::Not
    };

    preprocess_query(&mut q).unwrap();

    assert_eq!(q,
        Query::UnaryQuery{
            sub: Box::new(Query::FreetextQuery{
                tokens: Vec::default()
            }),
            op: UnaryOp::Not,
        }
    )
}

#[test]
fn test_struct_query(){

    let mut q = 
     
    Query::StructureQuery{
        sub: Box::new(Query::FreetextQuery{
            tokens: vec!["the".to_string()]
        }),
        elem: StructureElem::Category
    };

    preprocess_query(&mut q).unwrap();

    assert_eq!(q,
        Query::StructureQuery{
            sub: Box::new(Query::FreetextQuery{
                tokens: Vec::default()
            }),
            elem: StructureElem::Category
        }
    )
}

#[test]
fn test_relational_query(){

    let mut q = 
     
    Query::RelationQuery{
        sub: Some(Box::new(Query::FreetextQuery{
            tokens: vec!["the".to_string()]
        })),
        root: "AasdaSD ASDASd".to_string(),
        hops: 2
    };

    preprocess_query(&mut q).unwrap();

    assert_eq!(q,
        Query::RelationQuery{
            sub: Some(Box::new(Query::FreetextQuery{
                tokens: Vec::default()
            })),
            root: "AasdaSD ASDASd".to_string(), // cannot be preprocessed
            hops: 2,
        }
    )
}

#[test]
fn test_distance_query(){
    let mut q = 
     
    Query::DistanceQuery{
        lhs: "worm".to_string(),
        dst: 2,
        rhs: "bars".to_string(),
    };

    preprocess_query(&mut q).unwrap();

    assert_eq!(q,
        Query::DistanceQuery{
            lhs: "worm".to_string(),
            dst: 2,
            rhs: "bar".to_string(),
        }
    )
}

#[test]
#[should_panic]
fn test_distance_query_error(){
    let mut q = 
     
    Query::DistanceQuery{
        lhs: "the".to_string(),
        dst: 2,
        rhs: "bars".to_string(),
    };

    preprocess_query(&mut q).unwrap();
}

#[test]
fn test_phrase_query(){
    let mut q = 
     
    Query::PhraseQuery{
        tks: vec!["the".to_string(),"bikes".to_string()],
    };

    preprocess_query(&mut q).unwrap();

    assert_eq!(q,
        Query::PhraseQuery{
            tks: vec!["bike".to_string()],
        }
    )
}

#[test]
fn test_wildcard_query(){
    let mut q = 
     
    Query::WildcardQuery{
        prefix: "the".to_string(),
        postfix: "bArs".to_string(),
    };

    preprocess_query(&mut q).unwrap();

    assert_eq!(q,
        Query::WildcardQuery{
            prefix: "the".to_string(),
            postfix: "bars".to_string(),
        }
    )
}