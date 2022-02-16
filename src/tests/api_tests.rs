use crate::api::endpoints::search;

#[test]
fn test_api_search() {
    let x = parse_query("hello");
    println!("{:?}", x.unwrap());
}
