use crate::parser::ast::{Query,StructureElem};
use nom::{
    IResult,
    bytes::complete::{take_while1,tag_no_case,tag},
    multi::{separated_list0,many1},
    character::{is_alphanumeric,is_space},
    branch::{alt},
};

pub fn parse_query(nxt : &str) -> IResult<&str, Box<Query>> {
    alt((
        parse_structure_query,
        parse_freetext_query,
    ))(nxt)
}


pub fn parse_structure_query(nxt : &str) -> IResult<&str, Box<Query>>{
    
    let (nxt, struct_elem) = parse_structure_elem(nxt)?;
    let (nxt, _) = parse_whitespace(nxt)?;
    let (nxt, query) = parse_query(nxt)?;

    Ok((nxt,
        Box::new(Query::StructureQuery{
            elem: struct_elem,
            sub: query,
        }
    )))
}

pub fn parse_freetext_query(nxt : &str) -> IResult<&str, Box<Query>> {
    separated_list0(many1(tag(" ")), parse_token)(nxt)
    .map(|(nxt,res)| (nxt, Box::new(Query::FreetextQuery{
        tokens: res
    })))
}

pub fn parse_structure_elem(nxt : &str) -> IResult<&str,StructureElem>{
    alt((tag_no_case("#TITLE"),
        tag_no_case("#CATEGORY"),
        tag_no_case("#CITATION")))(nxt) // TODO: also recognize any '#<infobox name>' names
    .map(|(nxt,res)| (nxt, res.into())) 
}

pub fn parse_token(nxt : &str) -> IResult<&str, String> {
    take_while1(is_token_char)(nxt)
        .map(|(nxt,res)| (nxt, res.to_string()))
}


pub fn is_token_char(nxt :char) -> bool{
    return is_alphanumeric(nxt as u8) 
            || nxt == '%'
            || nxt == '&' 
            || nxt == '_';
}

pub fn parse_whitespace(nxt : &str) -> IResult<&str, &str> {
    take_while1(is_whitespace)(nxt)
}

pub fn is_whitespace(nxt : char) -> bool {
    return is_space(nxt as u8);
}