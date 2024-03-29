use crate::{get_document_with_date_time, PreIndex, DATE_TIME_FORMAT};

use crate::utils::{get_document_with_links, get_document_with_text};
use crate::{
    index::Index,
    index_structs::{LastUpdatedDate, Posting},
};
use chrono::NaiveDateTime;
use std::array::IntoIter;
use std::collections::HashMap;
use utils::utils::MemFootprintCalculator;

// TODO: split tests by library
// add integration tests

#[test]
fn test_real_mem_primitives() {
    assert_eq!((0 as u32).real_mem(), 4);
    assert_eq!((0 as u64).real_mem(), 8);

    assert_eq!(("hello".to_string()).real_mem(), 24 + 5);
    assert_eq!(("hello").real_mem(), 16 + 5);
    assert_eq!(
        (vec!["hello".to_string(), "hello".to_string()]).real_mem(),
        24 + 24 + 24 + 5 + 5
    );
    assert_eq!(
        HashMap::<_, _>::from_iter(IntoIter::new([(1 as u32, 2 as u32), (3, 4)])).real_mem(),
        4 * 4 + 48
    );
}
#[test]
fn test_real_mem_naive_date_time() {
    let doc_datetime = LastUpdatedDate {
        date_time: NaiveDateTime::parse_from_str("2015-07-01 08:59:60", "%Y-%m-%d %H:%M:%S")
            .unwrap(),
    };
    assert_eq!(doc_datetime.real_mem(), 4 + 4 + 4);
}
#[test]
fn test_index_date_time_parsing_correct() {
    let mut pre_idx = PreIndex::default();
    //Ideal test
    let str1 = "2015-07-01 08:59:60";
    //Shouldn't ideally happen, but testing if 0s are left out
    let test_str2 = "2015-7-1 8:9:6";
    let actual_str2 = "2015-7-1 08:09:06";

    //First example
    pre_idx
        .add_document(get_document_with_date_time(1, "1", str1))
        .unwrap();

    let datetime_correct1 = LastUpdatedDate {
        date_time: NaiveDateTime::parse_from_str(str1, DATE_TIME_FORMAT).unwrap(),
    };
    //Second Example

    pre_idx
        .add_document(get_document_with_date_time(2, "2", test_str2))
        .unwrap();
    let datetime_correct2 = LastUpdatedDate {
        date_time: NaiveDateTime::parse_from_str(actual_str2, DATE_TIME_FORMAT).unwrap(),
    };

    let idx = Index::from_pre_index(pre_idx);

    assert_eq!(idx.get_last_updated_date(1), Some(datetime_correct1));
    assert_eq!(idx.get_last_updated_date(2), Some(datetime_correct2));
}

#[test]
fn test_index_date_time_parsing_incorrect() {
    let mut pre_idx = PreIndex::default();

    let incorrect_str1 = "2015-07-01 08-59-60"; //Incorrect formatting
    let incorrect_str2 = "";
    let incorrect_str3 = "2015-07-01";
    let incorrect_str4 = "08:59:60";
    let incorrect_str5 = "9999-99-99 99:99:99";
    let incorrect_str6 = "0000-00-00 00:00:00";

    pre_idx
        .add_document(get_document_with_date_time(1, "1", incorrect_str1))
        .unwrap();
    pre_idx
        .add_document(get_document_with_date_time(2, "2", incorrect_str2))
        .unwrap();
    pre_idx
        .add_document(get_document_with_date_time(3, "3", incorrect_str3))
        .unwrap();
    pre_idx
        .add_document(get_document_with_date_time(4, "4", incorrect_str4))
        .unwrap();
    pre_idx
        .add_document(get_document_with_date_time(5, "5", incorrect_str5))
        .unwrap();
    pre_idx
        .add_document(get_document_with_date_time(6, "6", incorrect_str6))
        .unwrap();

    let idx = Index::default();

    for i in 1..7 {
        assert_eq!(idx.get_last_updated_date(i), None);
    }
}

#[test]
fn test_add_duplicate_article() {
    let mut pre_idx = PreIndex::default();

    pre_idx
        .add_document(get_document_with_text(
            2,
            "d0",
            vec![("", "aaa bbb")],
            "ccc ddd",
            vec!["eee fff"],
            "ggg hhh",
        ))
        .unwrap();

    let res = pre_idx.add_document(get_document_with_text(
        2,
        "d0",
        vec![("", "aaa bbb")],
        "ccc ddd",
        vec!["eee fff"],
        "ggg hhh",
    ));

    assert_eq!(res.is_err(), true);
}

#[test]
fn test_basic_index_get_postings() {
    let mut pre_idx = PreIndex::default();

    pre_idx
        .add_document(get_document_with_text(
            2,
            "d0",
            vec![("", "aaa bbb")],
            "ccc ddd",
            vec!["eee fff"],
            "ggg hhh",
        ))
        .unwrap();

    let idx = Index::from_pre_index(pre_idx);

    assert_eq!(
        idx.get_postings("aaa")
            .unwrap()
            .lock()
            .get()
            .unwrap()
            .postings
            .into_iter()
            .collect::<Vec<Posting>>(),
        vec![Posting {
            document_id: 2,
            position: 0,
        }]
    );

    assert_eq!(
        idx.get_postings("ddd")
            .unwrap()
            .lock()
            .get()
            .unwrap()
            .postings
            .into_iter()
            .collect::<Vec<Posting>>(),
        vec![Posting {
            document_id: 2,
            position: 3,
        }]
    );

    assert_eq!(
        idx.get_postings("ggg")
            .unwrap()
            .lock()
            .get()
            .unwrap()
            .postings
            .into_iter()
            .collect::<Vec<Posting>>(),
        vec![Posting {
            document_id: 2,
            position: 6,
        }]
    );

    assert_eq!(
        idx.get_postings("dick").map(|v| v
            .lock()
            .get()
            .unwrap()
            .postings
            .into_iter()
            .collect::<Vec<Posting>>()),
        None
    );
}

#[test]
fn test_sorted_postings() {
    let mut pre_idx = PreIndex::default();


    pre_idx
        .add_document(get_document_with_text(
            2,
            "d0",
            vec![("", "ggg bbb")],
            "ccc ddd",
            vec!["eee fff"],
            "ggg hhh",
        ))
        .unwrap();

    pre_idx
        .add_document(get_document_with_text(
            3,
            "d1",
            vec![("", "ggg bbb")],
            "ccc ddd",
            vec!["eee fff"],
            "ggg hhh",
        ))
        .unwrap();

    let idx = Index::from_pre_index(pre_idx);

    assert_eq!(
        idx.get_postings("ggg")
            .unwrap()
            .lock()
            .get()
            .unwrap()
            .postings
            .into_iter()
            .collect::<Vec<Posting>>(),
        vec![
            Posting {
                document_id: 2,
                position: 0
            },
            Posting {
                document_id: 2,
                position: 6
            },
            Posting {
                document_id: 3,
                position: 0,
            },
            Posting {
                document_id: 3,
                position: 6,
            },
        ]
    );
}

#[test]
fn test_basic_index_tf() {
    let mut pre_idx = PreIndex::default();

    pre_idx
        .add_document(get_document_with_text(
            0,
            "d0",
            vec![("infobox", "hello world"), ("infobox2", "hello")],
            "eggs world",
            vec!["this that", "that", "eggs"],
            "hello world",
        ))
        .unwrap();

    let idx = Index::from_pre_index(pre_idx);

    assert_eq!(idx.tf("hello", 0), 3);
    assert_eq!(idx.tf("world", 0), 3);
    assert_eq!(idx.tf("this", 0), 1);
    assert_eq!(idx.tf("that", 0), 2);
    assert_eq!(idx.tf("eggs", 0), 2);
    assert_eq!(idx.tf("kirby", 0), 0);
}

#[test]
fn test_basic_index_df() {
    let mut pre_idx = PreIndex::default();

    pre_idx
        .add_document(get_document_with_text(
            0,
            "d0",
            vec![("infobox", "hello world"), ("infobox2", "hello")],
            "eggs world",
            vec!["this that", "that", "eggs"],
            "hello world",
        ))
        .unwrap();

    pre_idx
        .add_document(get_document_with_text(
            1,
            "d1",
            vec![("infobox", "aaa aa"), ("infobox2", "aaa")],
            "eggs world",
            vec!["aaa aa", "aaa", "aa"],
            "aaa aaa",
        ))
        .unwrap();

    let idx = Index::from_pre_index(pre_idx);

    assert_eq!(idx.df("hello"), 1);
    assert_eq!(idx.df("world"), 2);
    assert_eq!(idx.df("this"), 1);
    assert_eq!(idx.df("that"), 1);
    assert_eq!(idx.df("eggs"), 2);
    assert_eq!(idx.df("kirby"), 0);
}

#[test]
fn test_basic_index_links() {
    let mut pre_idx = PreIndex::default();

    pre_idx
        .add_document(get_document_with_links(0, "source", "1\t2"))
        .unwrap();
    pre_idx
        .add_document(get_document_with_links(1, "target1", "2\t1"))
        .unwrap();
    pre_idx
        .add_document(get_document_with_links(2, "target2", "0\t1"))
        .unwrap();

    let idx = Index::from_pre_index(pre_idx);

    assert_eq!(idx.get_links(0), vec![1, 2]);
    assert_eq!(idx.get_links(1), vec![1, 2]);
    assert_eq!(idx.get_links(2), vec![0, 1]);

    assert_eq!(idx.get_incoming_links(0), vec![2]);
    assert_eq!(idx.get_incoming_links(1), vec![0, 1, 2]);
    assert_eq!(idx.get_incoming_links(2), vec![0, 1]);

    // assert_eq!(idx.id_to_title(0), Some(&"source".to_string()));
    // assert_eq!(idx.title_to_id("source".to_string()), Some(0));

    // assert_eq!(idx.id_to_title(1), Some(&"target1".to_string()));
    // assert_eq!(idx.title_to_id("target1".to_string()), Some(1));

    // assert_eq!(idx.id_to_title(2), Some(&"target2".to_string()));
    // assert_eq!(idx.title_to_id("target2".to_string()), Some(2));
}

// make_sure_postings_are_in_order(){
//    todo!();
//}

//make_sure_tokens_are_sorted(){
//     todo!()
// }
