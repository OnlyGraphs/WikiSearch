use crate::index::index::{Index};
use crate::index::index_builder::{IndexBuilder, SqlIndexBuilder};
use api_rs::wiki_search::{wiki_search_server::WikiSearch, CheckIndexReply, CheckIndexRequest};
use std::env;
use std::sync::{Arc, RwLock};
use tonic::{Request, Response, Status};

//The implementation listens to the scheduler and updates the index by checking against the dump id.

#[derive(Debug)]
pub struct CheckIndexService {
    pub index: Arc<RwLock<Box<dyn Index>>>,
}

#[tonic::async_trait]
impl WikiSearch for CheckIndexService {
    async fn update_index(
        &self,
        _request: Request<CheckIndexRequest>,
    ) -> Result<Response<CheckIndexReply>, Status> {

        let connection_string: String = env::var("DATABASE_URL").expect("Did not set URL.");

        let dump_id = match self.index.try_read(){
                Ok(v) => (v).get_dump_id(),
                Err(_) => 0
        };

        let index_builder = SqlIndexBuilder {
            connection_string: connection_string,
            dump_id: self.index.try_read().unwrap().get_dump_id(),
        };
        

        let res = match index_builder.build_index().await {
            Ok(v) => v,
            Err(e) => {
                return Ok(Response::new(CheckIndexReply {
                    success: false,
                    err_code: format!("{:?}", e),
                }))
            }
        };

        let mut guard = match self.index.try_write() {
            Ok(v) => v,
            Err(e) => {
                return Ok(Response::new(CheckIndexReply {
                    success: false,
                    err_code: e.to_string(),
                }))
            }
        };
        println!("{:?}",res);

        *guard = res;

        Ok(Response::new(CheckIndexReply {
            success: true,
            err_code: "".to_string(),
        }))
    }
}
