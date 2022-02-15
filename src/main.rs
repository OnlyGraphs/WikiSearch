mod api;
mod grpc_server;
mod index;
mod tests;
mod parser;
mod utils;

use crate::index::collections::SmallPostingMap;
use actix_cors::Cors;
use actix_files::Files;
use actix_web::{App, HttpServer};
use api_rs::wiki_search::{
    wiki_search_server::{WikiSearch, WikiSearchServer},
    CheckIndexRequest,
};
use grpc_server::CheckIndexService;
use index::index::{BasicIndex, Index};
use log::{error, info};
use pretty_env_logger;
use std::process;
use std::{
    env,
    io::{Error, ErrorKind},
    sync::{Arc, RwLock},
    thread,
};
use tonic::{transport::Server, Request};

const DEFAULT_GRPC_ADDRESS: &str = "127.0.0.1:50051";
const DEFAULT_REST_IP: &str = "127.0.0.1";
const DEFAULT_REST_PORT: &str = "8000";
const DEFAULT_STATICFILES_DIR: &str = "./staticfiles";

fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    let connection_string = env::var("DATABASE_URL")
        .unwrap_or_else(|_| {
            println!("Did not set DATABASE_URL.");
            process::exit(1);
        })
        .to_string();
    let grpc_address = env::var("GRPC_ADDRESS").unwrap_or(DEFAULT_GRPC_ADDRESS.to_string());
    let rest_ip: String = env::var("SEARCH_IP").unwrap_or(DEFAULT_REST_IP.to_string());
    let rest_port = env::var("SEARCH_PORT").unwrap_or(DEFAULT_REST_PORT.to_string());
    let static_serve_dir = env::var("STATIC_DIR").unwrap_or(DEFAULT_STATICFILES_DIR.to_string());

    // create shared memory for index
    let index: Arc<RwLock<Box<dyn Index>>> = Arc::new(RwLock::new(Box::new(BasicIndex::<
        SmallPostingMap,
    >::default())));

    // the rust docs seemed to perform multiple joins
    // with redeclarations of the handle, no idea if any version of that would work
    thread::spawn(move || {
        let mut retries = 3;
        while retries > 0 {
            let status = run_grpc(
                index.clone(),
                grpc_address.clone(),
                connection_string.clone(),
            );

            if status.is_err() {
                error!("GRPC server error: {:?}", status.err().unwrap());
                error!("GRPC server failed, restarting..");
                retries -= 1;
                if retries <= 0 {
                    error!("Retried 3 times, GRPC offline.");
                }
            } else {
                break;
            }
        }
    });

    let handle = thread::spawn(move || {
        let mut retries = 3;
        while retries > 0 {
            let status = run_rest(rest_ip.clone(), rest_port.clone(), static_serve_dir.clone());
            if status.is_err() {
                error!("REST service error: {:?}", status.err().unwrap());
                error!("REST service failed, restarting..");
                retries -= 1;
                if retries <= 0 {
                    error!("Retried 3 times, REST service offline");
                }
            } else {
                break;
            }
        }
    });

    handle
        .join()
        .map_err(|_e| Error::new(ErrorKind::Other, "Failed to join handle"))?;

    Ok(())
}

#[actix_web::main]
async fn run_grpc<'a>(
    index: Arc<RwLock<Box<dyn Index>>>,
    grpc_address: String,
    connection_string: String,
) -> std::io::Result<()> {
    // launc grpc serices and server
    info!("Lauching gRPC server");
    info!("Binding to {}", grpc_address);

    // build initial index
    let service = CheckIndexService {
        index: index.clone(),
        connection_string: connection_string,
    };

    info!("Building initial index..");

    let response = service
        .update_index(Request::new(CheckIndexRequest {}))
        .await
        .map_err(|c| Error::new(ErrorKind::Other, c))?;

    if !response.get_ref().success {
        return Err(Error::new(
            ErrorKind::Other,
            response.get_ref().err_code.to_string(),
        ));
    }

    info!("Built initial index.");
    // show it or error
    info!(
        "{:?}",
        index
            .try_read()
            .map_err(|c| Error::new(ErrorKind::Other, c.to_string()))
    );

    Server::builder()
        .add_service(WikiSearchServer::new(service))
        .serve(grpc_address.parse().unwrap())
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e))?;

    Ok(())
}

#[actix_web::main]
async fn run_rest(ip: String, port: String, static_dir: String) -> std::io::Result<()> {
    // launch REST api
    info!("Lauching Search API");
    let bind_address = format!("{}:{}", ip, port);

    info!("Binding to: {}", bind_address);

    HttpServer::new(move || {
        let cors = Cors::permissive();

        let static_dir_cpy = &static_dir;
        App::new()
            .wrap(cors)
            .service(api::endpoints::search)
            .service(api::endpoints::relational)
            .service(api::endpoints::feedback)
            .service(
                Files::new("/", static_dir_cpy)
                    .prefer_utf8(true)
                    .index_file("index.html"),
            )
    })
    .bind(bind_address)?
    .run()
    .await?;

    Ok(())
}
