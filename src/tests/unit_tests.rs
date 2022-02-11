use crate::index::index_builder::{IndexBuilder, SqlIndexBuilder};
use crate::index::{
    index::{BasicIndex, Index},
    index_structs::{PosRange, Posting},
};
use crate::tests::test_utils::{get_document_with_links, get_document_with_text};
use std::env;

#[test]
fn test_basic_index_get_postings() {
    let mut idx = BasicIndex::default();

    idx.add_document(get_document_with_text(
        2,
        "d0",
        vec![("", "aaa bbb")],
        "ccc ddd",
        vec!["eee fff"],
        "ggg hhh",
    ));

    assert_eq!(
        *idx.get_postings("aaa").unwrap(),
        vec![Posting {
            document_id: 2,
            position: 0,
        }]
    );

    assert_eq!(
        *idx.get_postings("ddd").unwrap(),
        vec![Posting {
            document_id: 2,
            position: 3,
        }]
    );

    assert_eq!(
        *idx.get_postings("ggg").unwrap(),
        vec![Posting {
            document_id: 2,
            position: 6,
        }]
    );

    assert_eq!(idx.get_postings("dick"), None);
}

#[test]
fn test_basic_index_get_extent() {
    let mut idx = BasicIndex::default();

    idx.add_document(get_document_with_text(
        2,
        "d0",
        vec![("infobox", "aaa bbb"), ("infobox2", "hello")],
        "ccc ddd",
        vec!["eee fff", "world", "eggs"],
        "ggg hhh",
    ));

    assert_eq!(
        *idx.get_extent_for("infobox", &2).unwrap(),
        PosRange {
            start_pos: 0,
            end_pos: 2,
        }
    );

    assert_eq!(
        *idx.get_extent_for("infobox2", &2).unwrap(),
        PosRange {
            start_pos: 2,
            end_pos: 3,
        }
    );

    assert_eq!(
        *idx.get_extent_for("citation", &2).unwrap(),
        PosRange {
            start_pos: 5,
            end_pos: 9
        }
    );

    assert_eq!(
        *idx.get_extent_for("categories", &2).unwrap(),
        PosRange {
            start_pos: 9,
            end_pos: 11
        }
    );

    assert_eq!(idx.get_extent_for("asd", &2), None);
}

#[test]
fn test_basic_index_tf() {
    let mut idx = BasicIndex::default();

    idx.add_document(get_document_with_text(
        0,
        "d0",
        vec![("infobox", "hello world"), ("infobox2", "hello")],
        "eggs world",
        vec!["this that", "that", "eggs"],
        "hello world",
    ));

    assert_eq!(idx.tf("hello", 0), 3);
    assert_eq!(idx.tf("world", 0), 3);
    assert_eq!(idx.tf("this", 0), 1);
    assert_eq!(idx.tf("that", 0), 2);
    assert_eq!(idx.tf("eggs", 0), 2);
    assert_eq!(idx.tf("kirby", 0), 0);
}

#[test]
fn test_basic_index_df() {
    let mut idx = BasicIndex::default();

    idx.add_document(get_document_with_text(
        0,
        "d0",
        vec![("infobox", "hello world"), ("infobox2", "hello")],
        "eggs world",
        vec!["this that", "that", "eggs"],
        "hello world",
    ));
    idx.add_document(get_document_with_text(
        1,
        "d1",
        vec![("infobox", "aaa aa"), ("infobox2", "aaa")],
        "eggs world",
        vec!["aaa aa", "aaa", "aa"],
        "aaa aaa",
    ));

    idx.finalize();

    assert_eq!(idx.df("hello"), 1);
    assert_eq!(idx.df("world"), 2);
    assert_eq!(idx.df("this"), 1);
    assert_eq!(idx.df("that"), 1);
    assert_eq!(idx.df("eggs"), 2);
    assert_eq!(idx.df("kirby"), 0);
}

#[test]
fn test_basic_index_links() {
    let mut idx = BasicIndex::default();

    idx.add_document(get_document_with_links(0, "source", "target1,target2"));
    idx.add_document(get_document_with_links(1, "target1", "a"));
    idx.add_document(get_document_with_links(2, "target2", "source"));

    idx.finalize();

    assert_eq!(idx.get_links(0), vec![1, 2]);
    assert_eq!(idx.get_links(1).len(), 0);
    assert_eq!(idx.get_links(2), vec![0]);

    assert_eq!(idx.id_to_title(0), Some(&"source".to_string()));
    assert_eq!(idx.title_to_id("source".to_string()), Some(0));

    assert_eq!(idx.id_to_title(1), Some(&"target1".to_string()));
    assert_eq!(idx.title_to_id("target1".to_string()), Some(1));

    assert_eq!(idx.id_to_title(2), Some(&"target2".to_string()));
    assert_eq!(idx.title_to_id("target2".to_string()), Some(2));
}

// #[test]
// fn test_arbitrary_index_example() {
//     let mut idx = Index::default();
//     let mut citations_text = Vec::new();
//     citations_text.push("author jayden will smith".to_string());
//     let mut citations_ids = Vec::new();
//     citations_ids.push(1);
//     let mut infobox_text = Vec::new();
//     infobox_text.push("kyle loves rust".to_string());
//     let mut infobox_ids = Vec::new();
//     infobox_ids.push(1);
//     let main_text = "april fourth month year julian gregorian calendars march months days april begins day week july additionally january leap years april ends day week december april flowers sweet pea daisy birthstone diamond meaning diamond innocence month file colorful spring garden jpg thumb px spring flowers".to_string();

//     let all_text = infobox_text.get(0).unwrap().to_owned()
//         + &main_text
//         + &citations_text.get(0).unwrap().to_owned()
//         + "month, dates, calendar";
//     let mut split = all_text.split(" ");

//     let tokens: Vec<&str> = split.collect();

//     println!("{:#?}", tokens.get(50));

//     let new_document = Document{
//         doc_id: 1,
//         title: "April".to_string(),
//         categories: "month,dates,calendar".to_string(),
//         last_updated_date: "2020-1-23 14-00-00".to_string(),
//         namespace: 0,
//         article_abstract: "April is the fourth month of the year in the Julian and Gregorian calendars and comes between March and May".to_string(),
//         infobox_text: infobox_text,
//         infobox_type: "testinfobox".to_string(),
//         infobox_ids:infobox_ids,
//         main_text: main_text,
//         article_links: " Maine,Copenhagen,September,Rotterdam,Harry S. Truman,January 1,Albert Hofmann,Buddhism,Florida,Calendar,Swaziland,2001,Netherlands,King,Mexico,Tanganyika,Haile Selassie,Buddha,1865,1971,Abraham Lincoln,monarch,Thomas Jefferson,1968,John Wilkes Booth,Willem-Alexander of the Netherlands,Pablo Picasso,Zurich,Benito Mussolini,South America,same-sex marriage,1973,Henry VIII of England,November,Gregorian calendar,Sweet Pea,Jacob Roggeveen,flower,October,Laos,Tbilisi,Ridran,Dance,April 14,Iran,Army,calendar,Iceland,ANZAC Day,1909,Asteraceae,Russia,1533,1721,July,1994,sweet pea,China,2014,Catherine, Duchess of Cambridge,republic,Easter,Major League Baseball,1979,Pope,1841,Diamond,1937,April 18,Church of England,Marathon,Southeast Asia,Elba,Poland,common year,earthquake,love,May 20".to_string(),
//         citations_text: citations_text,
//         citations_ids: citations_ids,
//     };

//     idx.add_document(new_document);

//     println!("{:#?}", idx);
// }
// // fn check_no_spaces_as_tokens_please{};

// // fn check_no_punctuation_as_tokens_please{}

#[tokio::test]
async fn test_index_builder() {
    let connection_string: String = env::var("DATABASE_URL").expect("Did not set URL.");
    let dump_id = 1;
    let index_builder = SqlIndexBuilder {
        connection_string: connection_string,
        dump_id: dump_id,
    };
    let mut idx = match index_builder.build_index().await {
        Ok(v) => v,
        Err(e) => panic!("{:#?}", e),
    };

    println!("{:?}", idx.df("april"));
    println!("{:?}", idx);
}

// //Check if index updates if dump id changes
// fn check_if_dump_id_changed() {
//     todo!();
// }

// fn check_if_dump_id_is_assigned() {
//     todo!();
// }

// fn check_word_pos_if_correct() {
//     todo!();
// }

// fn check_ids_same_len_as_body_extent_types() {
//     todo!();
// }

//////////////////// Utils Test ///////////////////////
