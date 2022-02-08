use crate::ast_parser::enums::OpNodeType;
use crate::ast_parser::enums::StructureNodeType;
use nom::{
    IResult,
    combinator::map_res,
    bytes::complete::{tag, take_while},
    character::complete::{char, alpha0, digit0},
    character::{is_alphabetic, is_digit},
    Err
};
use nom;
use std::str::FromStr;


// Constants
const CATEGORY_TAG: &str = "#CATEGORY";
const TITLE_TAG: &str = "#TITLE";
const CITATION_TAG: &str = "#CITATION";
const INFO_BOX_TAG: &str = "#INFO_BOX_TEMPLATE_NAME";
const DIST_TAG: &str = "#DIST";
const LINKED_TO_TAG: &str = "#LINKEDTO";
const OR: &str = "OR";
const AND: &str = "AND";
const NOT: &str = "NOT";

pub struct Arena {
    nodes: Vec<Node>,
}


pub struct Node {
    parent: Option<NodeId>,
    previous_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,

    /// The actual data which will be stored within the tree
    pub data: String,
}

pub struct CategoryNode {
    node_type: StructureNodeType,
    parent: Option<NodeId>,
    previous_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
    first_child: Option<NodeId>
}

pub struct OpNode{
    node_type: OpNodeType,
    parent: Option<NodeId>,
    previous_sibling: Option<NodeId>,
    next_sibling: Option<NodeId>,
    first_child: Option<NodeId>,
    last_child: Option<NodeId>,
}

pub struct DistNode{
    parent: Option<Node>,
    previous_sibling: Option<Node>,
    next_sibling: Option<Node>,
    first_child: Option<Node>,
    last_child: Option<Node>,
    
    /// Instead of data, we store the desired distance
    pub dist: u32,
}

pub struct NodeId {
    index: usize,
}

pub fn add_leaf(data: String) -> Node {
    Node {
        data : data,
        next_sibling : None,
        parent : None,
        previous_sibling : None,
    }
}


fn hello_parser(i: &str) -> nom::IResult<&str, &str> {
	nom::bytes::complete::tag("hello")(i)
}


// }
// https://docs.rs/nom/6.1.2/nom/type.IResult.html
// https://github.com/Geal/nom/blob/main/doc/choosing_a_combinator.md
// <structure_elem> ::= `#TITLE`
// 			 | `#CATEGORY`
// 			 | `#CITATION`
//           | `#` INFOBOX_TEMPLATE_NAME


// fn parse_query(s : &str) -> nom::IResult<&str,>

// fn parse_structure_elem(s : &str) -> nom::IResult<&str,CategoryNode>{


// }



pub fn parse_term(s : &str) -> IResult<&str, Node> {

    let term = alpha0(s);

    match term {
        Ok(t) => return Ok((t.0, Node{
            parent: None,
            previous_sibling: None,
            next_sibling: None,
            data: t.1.to_string(),
        })),
        Err(e) => return Err(e),
    }

}

// TODO: Consider whitespace
pub fn parse_dist_query(s : &str) -> IResult<&str, DistNode> {
    // `#DIST` `,` <number> `,` <term> `,` <term>        # Distance search
    
    // `#DIST` `,` <number> `,` <term> `,` <term>        # Distance search

    let (nxt, _) = tag(DIST_TAG)(s)?;
    let (nxt, _)  = char(',')(nxt)?;
    let (nxt, d) = digit0(nxt)?;
    let (nxt, _)  = char(',')(nxt)?;
    let (nxt, t1) = parse_term(nxt)?;
    let (nxt, _)  = char(',')(nxt)?;
    let (nxt, t2) = parse_term(nxt)?;

    let dist_node = DistNode {
        parent: None,
        previous_sibling: None,
        next_sibling: None,
        first_child: Some(t1),
        last_child: Some(t2),
        dist: FromStr::from_str(d).unwrap(),
    };
    
    Ok((s, dist_node))
}