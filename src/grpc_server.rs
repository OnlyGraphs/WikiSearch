use crate::index::index::Index;
use crate::index::index_builder::{IndexBuilder, SqlIndexBuilder};
use api_rs::wiki_search::{wiki_search_server::WikiSearch, CheckIndexReply, CheckIndexRequest};
use log::info;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use tonic::{Request, Response, Status};

//The implementation listens to the scheduler and updates the index by checking against the dump id.

#[derive(Debug)]
pub struct CheckIndexService {
    pub index: Arc<RwLock<Box<dyn Index>>>,
    pub connection_string: String,
}

#[tonic::async_trait]
impl WikiSearch for CheckIndexService {
    async fn update_index(
        &self,
        _request: Request<CheckIndexRequest>,
    ) -> Result<Response<CheckIndexReply>, Status> {
        info!("Received index build signal.");

        let index_builder = SqlIndexBuilder {
            connection_string: self.connection_string.clone(),
            dump_id: match self.index.try_read() {
                Ok(v) => v,
                Err(e) => {
                    return Ok(Response::new(CheckIndexReply {
                        success: false,
                        err_code: e.to_string(),
                    }))
                }
            }
            .get_dump_id(),
        };

        let timer = Instant::now();

        let res = match index_builder.build_index_if_needed().await {
            Ok(v) => v,
            Err(e) => {
                return Ok(Response::new(CheckIndexReply {
                    success: false,
                    err_code: format!("{:?}", e),
                }))
            }
        };

        let rebuilt = res.is_some();

        if !rebuilt {
            info!("Index is already up to date. Not rebuilding.");
            return Ok(Response::new(CheckIndexReply {
                success: true,
                err_code: "".to_string(),
            }));
        }

        let time = timer.elapsed();
        info!("Building index took {:?}", time);

        let mut guard = match self.index.try_write() {
            Ok(v) => v,
            Err(e) => {
                return Ok(Response::new(CheckIndexReply {
                    success: false,
                    err_code: e.to_string(),
                }))
            }
        };

        *guard = res.expect("Something impossible happened!");

        Ok(Response::new(CheckIndexReply {
            success: true,
            err_code: "".to_string(),
        }))
    }
}
