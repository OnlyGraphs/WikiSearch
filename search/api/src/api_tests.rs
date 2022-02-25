use crate::api::endpoints::search;
use crate::api::structs::RESTSearchData;
use crate::index::{
    collections::SmallPostingMap,
    index::{BasicIndex, Index},
};
use crate::test_utils::get_document_with_text;
use std::process;
use std::{
    env,
    io::{Error, ErrorKind},
    sync::{Arc, RwLock},
    thread,
};

use actix_web::web;
use actix_web::{test, App};

// Helper functions for test functions
fn setup_connection() -> String {
    let connection_string = env::var("DATABASE_URL")
        .unwrap_or_else(|_| {
            println!("Did not set DATABASE_URL.");
            process::exit(1);
        })
        .to_string();
    return connection_string;
}

//Function assumes index_tests.rs passed all tests
fn setup_arbitrary_index_pointer() -> Arc<RwLock<Box<dyn Index>>> {
    let mut idx = BasicIndex::<SmallPostingMap>::default();

    idx.add_document(get_document_with_text(
        0,
        "d0",
        vec![("infobox", "hello world"), ("infobox2", "hello")],
        "eggs world",
        vec!["this that", "that", "eggs"],
        "hello world",
    ))
    .unwrap();

    idx.add_document(get_document_with_text(
        1,
        "d1",
        vec![("infobox", "aaa aa"), ("infobox2", "aaa")],
        "eggs world",
        vec!["aaa aa", "aaa", "aa"],
        "aaa aaa",
    ))
    .unwrap();

    idx.finalize().unwrap();
    let index_pointer: Arc<RwLock<Box<dyn Index>>> = Arc::new(RwLock::new(Box::new(idx)));

    return index_pointer;
}

//Requires connection to Database
// #[actix_rt::test]
// async fn test_api_search_functionality() {
//     let data = RESTSearchData {
//         connection_string: setup_connection(),
//         index_rest: setup_arbitrary_index_pointer(),
//     };
//     let query = "Hello";
//     // let mut app = test::init_service(App::new().route("/", web::get().to(index))).await;

//     // let req = test::TestRequest::with_header("content-type", "/api/v1/search").to_http_request();
//     let mut app = test::init_service(
//         App::new()
//             .app_data(data)
//             .route("/api/v1/search", web::get().to(search)),
//     )
//     .await;
//     let req = test::TestRequest::get().uri("/").to_request();
//     // let resp: AppState = test::call_and_read_body_json(&mut app, req).await;

//     // let m = search(query, data);
//     // println!("{:?}", m);
// }
