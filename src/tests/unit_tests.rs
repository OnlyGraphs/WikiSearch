use crate::index::index_builder::IndexBuilder;
use crate::index::index_builder::SqlIndexBuilder;

use crate::BasicIndex;
use std::env;

#[test]
fn hello_world() {
    assert_eq!(2 + 2, 4);
}

#[tokio::test]
async fn create_arbitrary_index() {
    let connection_string: String = env::var("DATABASE_URL").expect("Did not set URL.");
    let dump_id = 1;
    let index_builder = SqlIndexBuilder {
        connection_string: connection_string,
        dump_id: dump_id,
    };
    let idx = match index_builder.build_index().await {
        Ok(v) => v,
        Err(e) => panic!("{:#?}", e),
    };

    println!("{:?}", idx.doc_freq.get("april"));
}

//Check if index updates if dump id changes
fn check_if_dump_id_changed() {
    todo!();
}

fn check_if_dump_id_is_assigned() {
    todo!();
}
