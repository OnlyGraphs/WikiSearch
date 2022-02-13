mod api;
mod grpc_server;
mod index;
mod tests;
mod parser;
mod utils;

use actix_cors::Cors;
use tonic::Request;
use actix_web::{App, HttpServer};
use actix_files::Files;
use api_rs::wiki_search::{wiki_search_server::{WikiSearchServer,WikiSearch},CheckIndexRequest};
use grpc_server::{CheckIndexService};
use index::index::{BasicIndex,Index};
use std::{
    env,
    io::{Error, ErrorKind},
    sync::{Arc, RwLock},
    thread,
};
use tonic::transport::Server;

fn main() -> std::io::Result<()> {

    
    // create shared memory for index
    let index : Arc<RwLock<Box<dyn Index>>> 
        = Arc::new(RwLock::new(Box::new(BasicIndex::default())));


    // the rust docs seemed to perform multiple joins
    // with redeclarations of the handle, no idea if any version of that would work
    thread::spawn(move || {
        run_grpc(index).expect("GRPC API Failed to run");
    });

    let handle = thread::spawn(move || {
        run_rest().expect("REST API Failed to run");
    });

    handle.join()
        .map_err(|_e| Error::new(ErrorKind::Other, "Failed to join handle"))?;

    Ok(())
}



#[actix_web::main]
async fn run_grpc(index: Arc<RwLock<Box<dyn Index>>>) -> std::io::Result<()> {
    // launc grpc serices and server
    println!("Lauching gRPC server");
    let grpc_address = env::var("GRPC_ADDRESS").unwrap_or("127.0.0.1:50051".to_string());
    
    println!("Binding to {}", grpc_address);


    // build initial index
    let service = CheckIndexService { index: index };
    service.update_index(Request::new(CheckIndexRequest{})).await.expect("Could not update the index initially");

    Server::builder()
        .add_service(WikiSearchServer::new(service))
        .serve(grpc_address.parse().unwrap())
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    Ok(())
}

#[actix_web::main]
async fn run_rest() -> std::io::Result<()> {
    // launch REST api
    println!("Lauching Search API");
    let ip: String = env::var("SEARCH_IP").unwrap_or("127.0.0.1".to_string());
    let port = env::var("SEARCH_PORT").unwrap_or("8000".to_string());
    let bind_address = format!("{}:{}", ip, port);
    let static_dir = env::var("STATIC_DIR").unwrap_or("./staticfiles".to_string());

    println!("Binding to: {}", bind_address);


    
    HttpServer::new(move || {
        let cors = Cors::permissive();

        let static_dir_cpy = &static_dir;
        App::new()
            .wrap(cors)
            .service(api::endpoints::search)
            .service(api::endpoints::relational)
            .service(api::endpoints::feedback)
            .service(Files::new("/", static_dir_cpy)
                .prefer_utf8(true)
                .index_file("index.html"))
    })
    .bind(bind_address)?
    .run()
    .await?;

    Ok(())
}
