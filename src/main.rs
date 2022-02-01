mod api;
mod grpc_server;
mod index;
mod tests;

use actix_web::{App, HttpServer};
use api_rs::wiki_search::wiki_search_server::WikiSearchServer;
use grpc_server::CheckIndexService;
use std::env;
use tonic::transport::Server;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // launc grpc serices and server
    println!("Lauching gRPC server");
    Server::builder()
        .add_service(WikiSearchServer::new(CheckIndexService::default()))
        .serve("[::1]:50051".parse().unwrap());

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
