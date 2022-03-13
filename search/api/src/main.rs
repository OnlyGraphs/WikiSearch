use actix_cors::Cors;
use actix_files::{Files, NamedFile};
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use actix_web::middleware::Logger;
use actix_web::{App, HttpServer};
use api_rs::wiki_search::{
    wiki_search_server::{WikiSearch, WikiSearchServer},
    CheckIndexRequest,
};
use core::time;
use index::index::Index;
use log::{error, info};
use pretty_env_logger;
use search_lib::endpoints;
use search_lib::grpc_server::CheckIndexService;
use search_lib::structs::RESTSearchData;
use std::path::Path;
use std::process;
use std::{
    env,
    fs::read_dir,
    io::{Error, ErrorKind},
    sync::{Arc, RwLock},
    thread,
};
use tonic::{transport::Server, Request};

const DEFAULT_STATICFILES_DIR: &str = "./staticfiles";
const DEFAULT_GRPC_ADDRESS: &str = "127.0.0.1:50051";
const DEFAULT_REST_IP: &str = "127.0.0.1";
const DEFAULT_REST_PORT: &str = "8000";

fn main() -> std::io::Result<()> {
    pretty_env_logger::init();

    let connection_string = env::var("DATABASE_URL")
        .unwrap_or_else(|_| {
            println!("Did not set DATABASE_URL.");
            process::exit(1);
        })
        .to_string();
    info!("Using DATABASE_URL: {:?}", connection_string);
    let grpc_address = env::var("GRPC_ADDRESS").unwrap_or(DEFAULT_GRPC_ADDRESS.to_string());
    let rest_ip: String = env::var("SEARCH_IP").unwrap_or(DEFAULT_REST_IP.to_string());
    let rest_port = env::var("SEARCH_PORT").unwrap_or(DEFAULT_REST_PORT.to_string());
    let static_serve_dir = env::var("STATIC_DIR").unwrap_or(DEFAULT_STATICFILES_DIR.to_string());

    info!("Reading Directories at STATIC_DIR:");
    read_dir(Path::new(&static_serve_dir))
        .unwrap()
        .map(|v| v.unwrap().path())
        .for_each(|v| info!("\t {}", v.display()));

    // create shared memory for index
    let index: Arc<RwLock<Index>> = Arc::new(RwLock::new(Index::default()));

    // the rust docs seemed to perform multiple joins
    // with redeclarations of the handle, no idea if any version of that would work
    let connection_string_grpc = connection_string.clone();
    let index_grpc = index.clone();
    thread::spawn(move || loop {
        let status = run_grpc(
            index_grpc.clone(),
            grpc_address.clone(),
            connection_string_grpc.clone(),
        );

        if status.is_err() {
            error!("GRPC server error: {:?}", status.err().unwrap());
            info!("GRPC server failed, restarting in 30s ..");
            thread::sleep(time::Duration::from_secs(30));
        } else {
            info!("GRPC server successfully shutdown.");
            break;
        }
    });
    let connection_string_rest = connection_string.clone();
    let index_rest = index.clone();
    let handle = thread::spawn(move || loop {
        let status = run_rest(
            rest_ip.clone(),
            rest_port.clone(),
            static_serve_dir.clone(),
            index_rest.clone(),
            connection_string_rest.clone(),
        );
        if status.is_err() {
            error!("REST service error: {:?}", status.err().unwrap());
            info!("REST service failed, restarting..");
            info!("REST service failed, retrying in 30s ..");
            thread::sleep(time::Duration::from_secs(30));
        } else {
            info!("REST service successfully shutdown.");
            break;
        }
    });

    handle
        .join()
        .map_err(|_e| Error::new(ErrorKind::Other, "Failed to join handle"))?;

    Ok(())
}

#[actix_web::main]
async fn run_grpc<'a>(
    index_grpc: Arc<RwLock<Index>>,
    grpc_address: String,
    connection_string: String,
) -> std::io::Result<()> {
    // launc grpc serices and server
    info!("Lauching gRPC server");
    info!("Binding to {}", grpc_address);

    // build initial index
    let service = CheckIndexService {
        index: index_grpc.clone(),
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
        index_grpc
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
async fn run_rest(
    ip: String,
    port: String,
    static_dir: String,
    index_rest: Arc<RwLock<Index>>,
    connection_string: String,
) -> std::io::Result<()> {
    // launch REST api
    info!("Lauching Search API");
    let bind_address = format!("{}:{}", ip, port);

    info!("Binding to: {}", bind_address);

    HttpServer::new(move || {
        let cors = Cors::permissive();
        let data = RESTSearchData {
            index_rest: index_rest.clone(),
            connection_string: connection_string.clone(),
        };
        let logger = Logger::default();

        App::new()
            .wrap(logger)
            .wrap(cors)
            .data(data)
            .service(endpoints::search)
            .service(endpoints::relational)
            .service(endpoints::feedback)
            .service(
                Files::new("/", static_dir.clone())
                    .prefer_utf8(true)
                    .index_file("index.html")
                    .default_handler(|req: ServiceRequest| {
                        let (http_req, _payload) = req.into_parts();
                        async {
                            let root = env::var("STATIC_DIR")
                                .unwrap_or(DEFAULT_STATICFILES_DIR.to_string()); // stupid af, can't just use the static_dir variable cuz of moves and lifetimes

                            let with_extension = format!("{}{}{}", root, http_req.path(), ".html");
                            let file = match NamedFile::open_async(with_extension.clone()).await {
                                Ok(v) => v,
                                Err(_e) => NamedFile::open_async(format!("{}/404.html", root))
                                    .await
                                    .expect(&format!("No file named 404.html at {}", root)),
                            };

                            let res = file.into_response(&http_req);
                            Ok(ServiceResponse::new(http_req, res))
                        }
                    }),
            )
    })
    .bind(bind_address)?
    .run()
    .await?;

    Ok(())
}
