use std::collections::HashSet;

use crate::ast::{BinaryOp, Query, StructureElem, UnaryOp};

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, tag_no_case, take_until, take_while, take_while1},
    character::complete::{digit0, digit1, anychar},
    character::{is_alphanumeric, is_space},
    combinator::{eof, not, peek},
    multi::{many1, separated_list0},
    sequence::terminated,
    IResult,
};

pub const DIST_TAG: &str = "#DIST";

// Helper functions

pub fn is_token_char(nxt: char) -> bool {
    return is_alphanumeric(nxt as u8) || nxt == '%' || nxt == '&' || nxt == '_';
}

pub fn parse_whitespace(nxt: &str) -> IResult<&str, &str> {
    take_while1(is_whitespace)(nxt)
}

pub fn parse_whitespace0(nxt: &str) -> IResult<&str, &str> {
    take_while(is_whitespace)(nxt)
}

pub fn is_whitespace(nxt: char) -> bool {
    return is_space(nxt as u8);
}


lazy_static!(
    static ref SEPS : HashSet<char> = HashSet::from_iter(
        vec!['?',',','\t','.','-','(',')','&','^','!','$','¥','€','¢','}','{','>',
        '<','@','+','÷','×','~','[',']','\\',':',';','=','_','`','|','•','√',
        'π','¶','∆','°','✓','™','®','©','%','\'']
        .into_iter());
);

pub fn is_seperator(nxt: char) -> bool {
    return is_space(nxt as u8) | SEPS.contains(&nxt)
    
}

// Parses any amount of whitespace, tab and comma separators
pub fn parse_separator(nxt: &str) -> IResult<&str, &str> {
    take_while(is_seperator)(nxt)
}

pub fn parse_separator1(nxt: &str) -> IResult<&str, &str> {
    take_while1(is_seperator)(nxt)
}

pub fn parse_separator_untill_eof(nxt: &str) -> IResult<&str, &str> {
    let (nxt, _) = take_while(is_seperator)(nxt)?;
    eof(nxt)
}

pub fn is_or(nxt: &str) -> bool {
    return nxt == "OR";
}

pub fn is_and(nxt: &str) -> bool {
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
    let (remainder, _) = parse_separator(nxt)?;
    
    if remainder.len() > 0 {
        return Err(nom::Err::Error(nom::error::Error::new(
            //the new struct, instead of the tuple
            "Too many elements. Not a dist query.",
            nom::error::ErrorKind::Tag,
        )));
    }

    let dst: u32;

    match d.parse::<u32>() {
        Ok(n) => dst = n,
        Err(_e) => {
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
    if nxt.chars().count() == 0 {
        return Err(nom::Err::Error(nom::error::Error::new(
            //the new struct, instead of the tuple
            "Empty query.",
            nom::error::ErrorKind::Tag,
        )));
    }

    alt((
        terminated(parse_dist_query, parse_separator_untill_eof),
        terminated(parse_relation_query, parse_separator_untill_eof),
        terminated(parse_structure_query, parse_separator_untill_eof),
        terminated(parse_not_query, parse_separator_untill_eof),
        terminated(parse_binary_query, parse_separator_untill_eof),
        terminated(parse_wildcard_query, parse_separator_untill_eof),
        terminated(parse_phrase_query, parse_separator_untill_eof),
        terminated(parse_freetext_query, parse_separator_untill_eof),
    ))(nxt)
}

fn parse_query_sub(nxt: &str) -> IResult<&str, Box<Query>> {
    if nxt.chars().count() == 0 {
        return Err(nom::Err::Error(nom::error::Error::new(
            //the new struct, instead of the tuple
            "Empty query.",
            nom::error::ErrorKind::Tag,
        )));
    }

    alt((
        parse_dist_query,
        parse_relation_query,
        parse_structure_query,
        parse_not_query,
        parse_binary_query,
        parse_wildcard_query,
        parse_phrase_query,
        parse_freetext_query,
    ))(nxt)
}

pub fn parse_structure_query(nxt: &str) -> IResult<&str, Box<Query>> {
    let (nxt, _) = parse_separator(nxt)?;
    let (nxt, struct_elem) = parse_structure_elem(nxt)?;

    if struct_elem == StructureElem::Infobox("dist".to_string()) {
        return Err(nom::Err::Error(nom::error::Error::new(
            //the new struct, instead of the tuple
            "detected DIST. Not a structure query",
            nom::error::ErrorKind::Tag,
        )));
    }

    let (nxt, _) = parse_separator(nxt)?;
    let (nxt, query) = parse_query_sub(nxt)?;

    Ok((
        nxt,
        Box::new(Query::StructureQuery {
            elem: struct_elem,
            sub: query,
        }),
    ))
}

// pub fn parse_relational_query(nxt: &str) -> IResult<&str, Box<Query>> {
//     //  `#LINKEDTO` `,` <multiple_terms> `,` <number> [`,` <query>]?
//     let (nxt, _) = tag_no_case("#LINKEDTO")(nxt)?;
//     let (nxt, _) = parse_separator(nxt)?;
//     let (nxt, mut id) = digit0(nxt)?;
//     let (nxt, _) = parse_separator(nxt)?;
//     let (nxt, hops) = digit0(nxt)?;
//     let (nxt, _) = parse_separator(nxt)?;
//     let res = opt(parse_query_sub)(nxt);

//     let sub_query = match res {
//         Ok((_, Some(v))) => match *v {
//             Query::FreetextQuery { tokens } if tokens.len() == 0 => None,
//             _ => Some(v),
//         },
//         _ => None,
//     };

//     Ok((
//         nxt,
//         Box::new(Query::RelationQuery {
//             root: id.parse().map_err(|_e| nom::Err::Error(nom::error::Error::new(hops, nom::error::ErrorKind::Digit)))?,
//             hops: hops.parse().map_err(|_e|nom::Err::Error(nom::error::Error::new(hops, nom::error::ErrorKind::Digit)))?,
//             sub: sub_query,
//         }),
//     ))
// }

pub fn parse_freetext_query(nxt: &str) -> IResult<&str, Box<Query>> {
    let (nxt, _) = parse_separator(nxt)?;
    separated_list0(parse_separator1, parse_token)(nxt)
        .map(|(nxt, res)| (nxt, Box::new(Query::FreetextQuery { tokens: res })))
}

pub fn parse_structure_elem(nxt: &str) -> IResult<&str, StructureElem> {
    let (nxt, _) = parse_separator(nxt)?;

    let (nxt, _) = tag("#")(nxt)?;
    alt((
        tag_no_case("TITLE"),
        tag_no_case("CATEGORY"),
        tag_no_case("CITATION"),
        parse_token_str,
    ))(nxt) // TODO: also recognize any '#<infobox name>' names
    .map(|(nxt, res)| (nxt, res.into()))
}

pub fn parse_token(nxt: &str) -> IResult<&str, String> {
    take_while1(is_token_char)(nxt).map(|(nxt, res)| (nxt, res.to_string()))
}

pub fn parse_token_str(nxt: &str) -> IResult<&str, &str> {
    take_while1(is_token_char)(nxt)
}

pub fn parse_token0(nxt: &str) -> IResult<&str, String> {
    take_while(is_token_char)(nxt).map(|(nxt, res)| (nxt, res.to_string()))
}

// pub fn parse_token_in_phrase(nxt: &str) -> IResult<&str, String> {
//     let (nxt, _) = parse_separator(nxt)?;
//     let (nxt, token) = parse_token(nxt)?;
//     let (nxt, _) = parse_separator(nxt)?;
//     Ok((nxt, token))
// }

// pub fn parse_page_title(nxt: &str) -> IResult<&str, String> {
//     let (nxt, _) = parse_whitespace0(nxt)?;
//     let (nxt, token) = parse_token(nxt)?;
//     let (nxt, _) = parse_whitespace0(nxt)?;
//     Ok((nxt, token))
// }

pub fn parse_not_query(nxt: &str) -> IResult<&str, Box<Query>> {
    let (nxt, _) = parse_separator(nxt)?;
    let (nxt, _) = tag_no_case("NOT")(nxt)?;
    let (nxt, _) = parse_separator1(nxt)?;
    let (nxt, query) = parse_query_sub(nxt)?;

    return Ok((
        nxt,
        Box::new(Query::UnaryQuery {
            op: UnaryOp::Not,
            sub: query,
        }),
    ));
}

// TODO: Make OR not case sensitive
pub fn parse_or_query(nxt: &str) -> IResult<&str, Box<Query>> {
    let (nxt, _) = parse_separator(nxt)?;
    let (query2, query1) = alt((
        take_until(" OR"),
        take_until(",OR")
    ))(nxt)?;
    let (query2, _) = anychar(query2)?;
    let (query2, _) = tag("OR")(query2)?;
    let (query2, _) = parse_separator1(query2)?;

    let (_nxt, q1) = parse_query_sub(query1)?;
    let (nxt, q2) = parse_query_sub(query2)?;

    return Ok((
        nxt,
        Box::new(Query::BinaryQuery {
            op: BinaryOp::Or,
            lhs: q1,
            rhs: q2,
        }),
    ));
}

// TODO: Make AND not case sensitive
pub fn parse_and_query(nxt: &str) -> IResult<&str, Box<Query>> {
    let (nxt, _) = parse_separator(nxt)?;
    let (query2, query1) = alt((
        take_until(" AND"),
        take_until(",AND")
    ))(nxt)?;

    let (query2, _) = anychar(query2)?;
    let (query2, _) = tag("AND")(query2)?;
    let (query2, _) = parse_separator1(query2)?;
    let (_nxt, q1) = parse_query_sub(query1)?;
    let (nxt, q2) = parse_query_sub(query2)?;

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

// TODO: Remove separators
pub fn parse_wildcard_query(nxt: &str) -> IResult<&str, Box<Query>> {
    let (nxt, _) = parse_separator(nxt)?;
    let (nxt, lhs) = parse_token0(nxt)?;
    let (nxt, _) = parse_separator(nxt)?;
    let (nxt, _) = tag("*")(nxt)?;
    let (nxt, _) = parse_separator(nxt)?;
    let (nxt, rhs) = parse_token0(nxt)?;

    Ok((
        nxt,
        Box::new(Query::WildcardQuery {
            prefix: lhs.to_string(),
            suffix: rhs.to_string(),
        }),
    ))
}

pub fn parse_simple_relation_query(nxt: &str) -> IResult<&str, Box<Query>> {
    let (nxt, _) = parse_separator(nxt)?;
    let (nxt, _) = tag_no_case("#LinksTo")(nxt)?;
    let (nxt, _) = parse_separator(nxt)?;
    let (nxt, id) = digit0(nxt)?;
    let (nxt, _) = parse_separator(nxt)?;
    let (nxt, hops) = digit1(nxt)?;

    Ok((
        nxt,
        Box::new(Query::RelationQuery {
            root: id.parse().map_err(|_e| {
                nom::Err::Error(nom::error::Error::new(hops, nom::error::ErrorKind::Digit))
            })?,
            hops: hops.parse().map_err(|_e| {
                nom::Err::Error(nom::error::Error::new(
                    "Cannot convert string containing distance to integer.",
                    nom::error::ErrorKind::Tag,
                ))
            })?,
            sub: None,
        }),
    ))
}

// TODO: What if the title contains integers?
pub fn parse_nested_relation_query(nxt: &str) -> IResult<&str, Box<Query>> {
    let (nxt, _) = parse_separator(nxt)?;
    let (nxt, _) = tag_no_case("#LinksTo")(nxt)?;
    let (nxt, _) = parse_separator(nxt)?;
    let (nxt, id) = digit0(nxt)?;
    let (nxt, _) = parse_separator(nxt)?;
    let (nxt, hops) = digit1(nxt)?;
    let (nxt, _) = parse_separator(nxt)?;
    let (nxt, sub) = parse_query_sub(nxt)?;

    Ok((
        nxt,
        Box::new(Query::RelationQuery {
            root: id.parse().map_err(|_e| {
                nom::Err::Error(nom::error::Error::new(hops, nom::error::ErrorKind::Digit))
            })?,
            hops: hops.parse().map_err(|_e| {
                nom::Err::Error(nom::error::Error::new(
                    "Cannot convert string containing distance to integer.",
                    nom::error::ErrorKind::Tag,
                ))
            })?,
            sub: Some(Box::new(*sub)),
        }),
    ))
}

pub fn parse_relation_query(nxt: &str) -> IResult<&str, Box<Query>> {
    let (nxt, _) = parse_separator(nxt)?;
    alt((parse_nested_relation_query, parse_simple_relation_query))(nxt)
}

pub fn parse_phrase_query(nxt: &str) -> IResult<&str, Box<Query>> {
    let (nxt, _) = parse_separator(nxt)?;
    let (nxt, _) = tag("\"")(nxt)?;
    let (nxt, _) = parse_separator(nxt)?;
    let (nxt, tokens) = separated_list0(parse_separator1, parse_token)(nxt)?;
    let (nxt, _) = parse_separator(nxt)?;
    let (nxt, _) = tag("\"")(nxt)?;

    Ok((nxt, Box::new(Query::PhraseQuery { tks: tokens })))
}
