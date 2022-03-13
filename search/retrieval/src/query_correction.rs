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

///main function to be called to spell check query
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

/// helper recursive function to perform spell checking
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

/// helper function to mark the tokens that have number of postings below threshold.
/// At the end, returns a vector of the tokens corrected
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
                .map(|v| v.lock().get().unwrap().postings_count);
            //Retrieve the condition by checking length of posting against threshold.
            //spell correct flag == true indicates that we will perform spell checking on this token. if false, we essentially dont correct it
            let spell_correct_flag: bool = len_posting.unwrap_or(0) < token_threshold;
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
/// Naive way of doing spell correction,
/// Finds tokens closest to the current passed token (key) in the ternary index tree
/// Returns either the current passed key (if nothing close to it was found), or returns the closest key to it
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

            //sort by length closest to the token.
            //Performs subtraction, but if something goes wrong, unwrap to default of substitution
            //TODO: maybe check to see what happens when subtraction results in integer overflows
            closest_keys.sort_by_key(|s| {
                i8::abs(
                    (s.len() as i8)
                        .checked_sub(token.len() as i8)
                        .unwrap_or_default(),
                )
            });

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
