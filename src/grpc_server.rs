use crate::index::index::Index;
use crate::index::index_builder::{IndexBuilder, SqlIndexBuilder};
use api_rs::wiki_search::{wiki_search_server::WikiSearch, CheckIndexReply, CheckIndexRequest};
use std::sync::{Arc, RwLock};
use tonic::{Request, Response, Status};
use log::{info,error};

//The implementation listens to the scheduler and updates the index by checking against the dump id.

#[derive(Debug)]
pub struct CheckIndexService {
    pub index: Arc<RwLock<Box<dyn Index>>>,
    pub connection_string: String
}

#[tonic::async_trait]
impl WikiSearch for CheckIndexService {
    async fn update_index(
        &self,
        _request: Request<CheckIndexRequest>,
    ) -> Result<Response<CheckIndexReply>, Status> {

        let index_builder = SqlIndexBuilder {
            connection_string: self.connection_string.clone(),
            dump_id: match self.index.try_read(){
                Ok(v) => v,
                Err(e) => return Ok(Response::new(CheckIndexReply {
                    success: false,
                    err_code: e.to_string(),
                }))
            }.get_dump_id(),
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

        *guard = res;

        Ok(Response::new(CheckIndexReply {
            success: true,
            err_code: "".to_string(),
        }))
    }
}
