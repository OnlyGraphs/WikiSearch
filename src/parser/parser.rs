use crate::parser::ast::{BinaryOp, Query, StructureElem, UnaryOp};

use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_until, take_while, take_while1},
    character::complete::digit0,
    character::{is_alphanumeric, is_space},
    multi::{separated_list0, many_m_n},
    combinator::{opt},
    IResult,
};
use std::str::FromStr;

const DIST_TAG: &str = "#DIST";

// Helper functions

pub fn is_token_char(nxt: char) -> bool {
    return is_alphanumeric(nxt as u8) || nxt == '%' || nxt == '&' || nxt == '_';
}

pub fn parse_whitespace(nxt: &str) -> IResult<&str, &str> {
    take_while1(is_whitespace)(nxt)
}

pub fn is_whitespace(nxt: char) -> bool {
    return is_space(nxt as u8);
}

pub fn is_comma(nxt: char) -> bool {
    return nxt == ',';
}

pub fn is_tab(nxt: char) -> bool {
    return nxt == '\t';
}

pub fn is_seperator(nxt: char) -> bool {
    return is_whitespace(nxt) | is_comma(nxt) | is_tab(nxt);
}

// Parses any amount of whitespace, tab and comma separators
pub fn parse_separator(nxt: &str) -> IResult<&str, &str> {
    take_while(is_seperator)(nxt)
}

pub fn is_OR(nxt: &str) -> bool {
    return nxt == "OR";
}

pub fn is_AND(nxt: &str) -> bool {
    return nxt == "AND";
}

// TODO: Consider more than single tokens (e.g.: #DIST,3,pumpkin pie,latte)
// Note that this only considers single tokens
pub fn parse_dist_query(nxt: &str) -> IResult<&str, Box<Query>> {
    // `#DIST` `,` <number> `,` <term> `,` <term>        # Distance search
    let (nxt, _) = parse_separator(nxt)?;
    let (nxt, _) = tag(DIST_TAG)(nxt)?;
    let (nxt, _) = parse_separator(nxt)?;
    let (nxt, d) = digit0(nxt)?;
    let (nxt, _) = parse_separator(nxt)?;
    let (nxt, t1) = parse_token(nxt)?;
    let (nxt, _) = parse_separator(nxt)?;
    let (nxt, t2) = parse_token(nxt)?;

    let dst: u32;

    match d.parse::<u32>() {
        Ok(n) => dst = n,
        Err(e) => {
            return Err(nom::Err::Error(nom::error::Error::new(
                //the new struct, instead of the tuple
                "Cannot convert string containing distance to integer.",
                nom::error::ErrorKind::Tag,
            )));
        }
    };

    let dist_query = Query::DistanceQuery {
        dst: dst,
        lhs: t1,
        rhs: t2,
    };

    Ok((nxt, Box::new(dist_query)))
}

pub fn parse_query(nxt: &str) -> IResult<&str, Box<Query>> {
    alt((
        parse_relational_query,
        parse_dist_query,
        parse_binary_query,
        parse_not_query,
        parse_structure_query,
        parse_freetext_query,
    ))(nxt)
}

pub fn parse_structure_query(nxt: &str) -> IResult<&str, Box<Query>> {
    let (nxt, struct_elem) = parse_structure_elem(nxt)?;
    let (nxt, _) = parse_whitespace(nxt)?;
    let (nxt, query) = parse_query(nxt)?;

    Ok((
        nxt,
        Box::new(Query::StructureQuery {
            elem: struct_elem,
            sub: query,
        }),
    ))
}


pub fn parse_relational_query(nxt: &str) -> IResult<&str, Box<Query>> {
    //  `#LINKEDTO` `,` <multiple_terms> `,` <number> [`,` <query>]?
    let (nxt, _) = tag_no_case("#LINKEDTO")(nxt)?;
    let (nxt, _) = parse_separator(nxt)?;
    let (nxt, mut title) = take_until(",")(nxt)?;
    title = title.trim();
    let (nxt, _) = parse_separator(nxt)?;
    let (nxt, hops) = digit0(nxt)?;
    let (nxt, _) = parse_separator(nxt)?;
    let res = opt(parse_query)(nxt);

    let sub_query = match res {
        Ok((_,Some(v))) => match *v {
            Query::FreetextQuery {tokens} if tokens.len() == 0 => None,
            _ => Some(v), 
        },
        _ => None  
    };

    Ok((
        nxt,
        Box::new(Query::RelationQuery {
            root: title.to_string(),
            hops: hops.parse().map_err(|e| nom::Err::Error(nom::error::Error::new(
                hops,
                nom::error::ErrorKind::Digit,
            )))?,
            sub: sub_query,
        }),
    ))
}

pub fn parse_freetext_query(nxt: &str) -> IResult<&str, Box<Query>> {
    separated_list0(parse_whitespace, parse_token)(nxt)
        .map(|(nxt, res)| (nxt, Box::new(Query::FreetextQuery { tokens: res })))
}

pub fn parse_structure_elem(nxt: &str) -> IResult<&str, StructureElem> {
    alt((
        tag_no_case("#TITLE"),
        tag_no_case("#CATEGORY"),
        tag_no_case("#CITATION"),
    ))(nxt) // TODO: also recognize any '#<infobox name>' names
    .map(|(nxt, res)| (nxt, res.into()))
}

pub fn parse_token(nxt: &str) -> IResult<&str, String> {
    take_while1(is_token_char)(nxt).map(|(nxt, res)| (nxt, res.to_string()))
}

pub fn parse_not_query(nxt: &str) -> IResult<&str, Box<Query>> {
    let (nxt, _) = parse_separator(nxt)?;
    let (nxt, _) = tag_no_case("NOT")(nxt)?;
    let (nxt, _) = parse_separator(nxt)?;
    let (nxt, query) = parse_query(nxt)?;

    return Ok((
        nxt,
        Box::new(Query::UnaryQuery {
            op: UnaryOp::Not,
            sub: query,
        }),
    ));
}

pub fn parse_or_query(nxt: &str) -> IResult<&str, Box<Query>> {
    let (nxt, _) = parse_separator(nxt)?;
    let (query2, query1) = take_until("OR")(nxt)?;
    let (query2, _) = tag("OR")(query2)?;
    let (query2, _) = parse_separator(query2)?;

    let (nxt, q1) = parse_query(query1)?;
    let (nxt, q2) = parse_query(query2)?;

    return Ok((
        nxt,
        Box::new(Query::BinaryQuery {
            op: BinaryOp::Or,
            lhs: q1,
            rhs: q2,
        }),
    ));
}

pub fn parse_and_query(nxt: &str) -> IResult<&str, Box<Query>> {
    let (nxt, _) = parse_separator(nxt)?;
    let (query2, query1) = take_until("AND")(nxt)?;
    let (query2, _) = tag("AND")(query2)?;
    let (query2, _) = parse_separator(query2)?;
    let (nxt, q1) = parse_query(query1)?;
    let (nxt, q2) = parse_query(query2)?;

    return Ok((
        nxt,
        Box::new(Query::BinaryQuery {
            op: BinaryOp::And,
            lhs: q1,
            rhs: q2,
        }),
    ));
}

pub fn parse_binary_query(nxt: &str) -> IResult<&str, Box<Query>> {
    alt((parse_and_query, parse_or_query))(nxt)
}
