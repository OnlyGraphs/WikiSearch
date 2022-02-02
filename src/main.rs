mod api;
mod grpc_server;
mod index;
mod tests;

use actix_web::{App, HttpServer};
use api_rs::wiki_search::wiki_search_server::WikiSearchServer;
use grpc_server::CheckIndexService;
use index::index::BasicIndex;
use std::env;
use std::sync::{Arc, Barrier, RwLock};
use std::thread;
use tonic::transport::Server;

fn main() -> std::io::Result<()> {
    let barrier = Arc::new(Barrier::new(2));

    // the rust docs seemed to perform multiple joins
    // with redeclarations of the handle, no idea if any version of that would work
    let handle = thread::spawn(move || {
        run_grpc();
    });

    let handle = thread::spawn(move || {
        run_rest();
    });

    handle.join();

    Ok(())
}

#[actix_web::main]
async fn run_grpc() -> std::io::Result<()> {
    // launc grpc serices and server
    println!("Lauching gRPC server");
    let grpc_address = env::var("GRPC_ADDRESS").unwrap_or("127.0.0.1:50051".to_string());
    println!("Binding to {}", grpc_address);

    // create shared memory for index
    let index = Arc::new(RwLock::new(BasicIndex::default()));

    Server::builder()
        .add_service(WikiSearchServer::new(CheckIndexService { index: index }))
        .serve(grpc_address.parse().unwrap())
        .await;

    Ok(())
}

#[actix_web::main]
async fn run_rest() -> std::io::Result<()> {
    // launch REST api
    println!("Lauching Search API");
    let ip: String = env::var("SEARCH_IP").unwrap_or("127.0.0.1".to_string());
    let port = env::var("SEARCH_PORT").unwrap_or("8000".to_string());
    let bind_address = format!("{}:{}", ip, port);

    println!("Binding to: {}", bind_address);
    HttpServer::new(|| {
        App::new()
            .service(api::endpoints::search)
            .service(api::endpoints::relational)
            .service(api::endpoints::feedback)
    })
    .bind(bind_address)?
    .run()
    .await;

    Ok(())
}
