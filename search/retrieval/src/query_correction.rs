use index::index::Index;

use log::info;
// use parser::errors::{QueryError, QueryErrorKind};
use parser::ast::Query;

// for search - query correction (spell correction)
pub const TOTAL_POSTING_CORRECTION_THRESHOLD: u32 = 10000; // If the results are below this threshold, we execute spell checking
pub const TOKEN_CORRECTION_THRESHOLD: u32 = 1000; //For each token, see if we perform spell checking
pub const CORRECTION_TRIES: u8 = 2; // Number of tries to attempt spell checking. for each failed try, the distance to key argument is increased
pub const CORRECTION_KEY_DISTANCE: u8 = 1; //Starting distance of current token that is being spell checked to closest words/neighbours in the tree. In orher words, how far do we look in the tree by the difference of characters
pub const CORRECTION_KEY_DISTANCE_ADD_PER_TRY: u8 = 1; //Increase in distance key per try iteration

pub fn correct_query<'a>(query: &Query, index: &'a Index) -> String {
    //Define parameters
    let token_correction_threshold: u32 = std::env::var("TOKEN_CORRECTION_THRESHOLD")
        .unwrap_or(TOKEN_CORRECTION_THRESHOLD.to_string())
        .parse()
        .unwrap();
    let correction_number_of_tries_per_token: u8 = std::env::var("CORRECTION_TRIES")
        .unwrap_or(CORRECTION_TRIES.to_string())
        .parse()
        .unwrap();
    let correction_key_distance_per_token: u8 = std::env::var("CORRECTION_KEY_DISTANCE")
        .unwrap_or(CORRECTION_KEY_DISTANCE.to_string())
        .parse()
        .unwrap();
    let correction_key_distance_per_token_append_amount: u8 =
        std::env::var("CORRECTION_KEY_DISTANCE_ADD_PER_TRY")
            .unwrap_or(CORRECTION_KEY_DISTANCE_ADD_PER_TRY.to_string())
            .parse()
            .unwrap();
    let new_query = correct_query_sub(
        query,
        index,
        token_correction_threshold,
        correction_number_of_tries_per_token,
        correction_key_distance_per_token,
        correction_key_distance_per_token_append_amount,
    );
    let mut suggestion = "".to_string();
    if new_query != *query {
        suggestion = format!("{}", new_query);
    }
    return suggestion;
}

pub fn correct_query_sub<'a>(
    query: &Query,
    index: &'a Index,
    token_threshold: u32,
    number_of_tries: u8,
    key_distance: u8,
    key_distance_append_amount: u8,
) -> Query {
    match query {
        Query::BinaryQuery { op, lhs, rhs } => {
            return Query::BinaryQuery {
                op: op.clone(),
                lhs: Box::new(correct_query_sub(
                    lhs,
                    index,
                    token_threshold,
                    number_of_tries,
                    key_distance,
                    key_distance_append_amount,
                )),
                rhs: Box::new(correct_query_sub(
                    rhs,
                    index,
                    token_threshold,
                    number_of_tries,
                    key_distance,
                    key_distance_append_amount,
                )),
            };
        }
        Query::UnaryQuery { op, ref sub } => {
            return Query::UnaryQuery {
                op: op.clone(),
                sub: Box::new(correct_query_sub(
                    sub,
                    index,
                    token_threshold,
                    number_of_tries,
                    key_distance,
                    key_distance_append_amount,
                )),
            };
        }
        Query::PhraseQuery { tks } => {
            let new_tokens = mark_tokens_to_correct(
                tks,
                index,
                token_threshold,
                number_of_tries,
                key_distance,
                key_distance_append_amount,
            );
            let new_query = Query::PhraseQuery { tks: new_tokens };
            return new_query;
        }
        Query::DistanceQuery { dst, lhs, rhs } => query.clone(),
        Query::StructureQuery { elem, sub } => query.clone(),
        Query::RelationQuery { root, hops, sub } => query.clone(),
        Query::WildcardQuery { prefix, suffix } => query.clone(),
        Query::FreetextQuery { tokens } => {
            let new_tokens = mark_tokens_to_correct(
                tokens,
                index,
                token_threshold,
                number_of_tries,
                key_distance,
                key_distance_append_amount,
            );

            let new_query = Query::FreetextQuery { tokens: new_tokens };
            return new_query;
        }
    }
}

fn mark_tokens_to_correct<'a>(
    tokens: &Vec<String>,
    index: &'a Index,
    token_threshold: u32,
    number_of_tries: u8,
    key_distance: u8,
    key_distance_append_amount: u8,
) -> Vec<String> {
    let mut new_tokens = Vec::<String>::new();
    let existing_tokens_markers: Vec<(&String, bool)> = tokens
        .iter()
        .map(|token| {
            let len_posting = index
                .get_postings(token)
                .map(|v| v.lock().get().unwrap().df);
            info!("len_posting {:?}", len_posting.unwrap_or(0));
            let spell_correct_flag = len_posting.unwrap_or(0) < token_threshold;
            (token, spell_correct_flag)
        })
        .collect::<Vec<(&String, bool)>>();

    for (token, spell_correct_flag) in existing_tokens_markers {
        if spell_correct_flag {
            let corrected_token = investigate_query_naive_correction(
                token,
                index,
                number_of_tries,
                key_distance,
                key_distance_append_amount,
            );
            new_tokens.push(corrected_token.clone());
        } else {
            new_tokens.push(token.clone());
        }
    }
    new_tokens
}
// Naive way of doing spell correction.
pub fn investigate_query_naive_correction<'a>(
    token: &'a String,
    index: &'a Index,
    mut tries: u8,
    mut key_distance: u8,
    key_distance_append_amount: u8,
) -> String {
    // let token = tokens.pop().unwrap_or("".to_string());

    while tries > 0 {
        if token != "" {
            let mut closest_keys = index
                .posting_nodes
                .find_nearest_neighbour_keys(&token, key_distance.into());
            info!("First key {:?}", closest_keys.get(0));
            info!("Second key {:?}", closest_keys.get(1));
            info!("Third key {:?}", closest_keys.get(2));
            info!("Fourth key {:?}", closest_keys.get(3));
            info!("SecoFifthnd key {:?}", closest_keys.get(4));
            //If the next functions panic, the token is huge

            closest_keys.sort_by_key(|s| {
                i16::abs(
                    (s.len() as i16)
                        .checked_sub(token.len() as i16)
                        .unwrap_or_default(),
                )
            });
            info!("First key {:?}", closest_keys.get(0));
            info!("Second key {:?}", closest_keys.get(1));
            info!("Third key {:?}", closest_keys.get(2));
            info!("Fourth key {:?}", closest_keys.get(3));
            info!("Fifth key {:?}", closest_keys.get(4));

            if !closest_keys.is_empty() {
                return closest_keys.get(0).unwrap_or(token).to_string();
            } else {
                tries -= 1;
                key_distance += key_distance_append_amount;
            }
        }
    }
    return token.to_string();
}
