mod api;
mod tests;
mod index;

use actix_web::{App, HttpServer};
use std::env;
use index::index_builder::IndexBuilder;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let connection_string: String = env::var("DATABASE_URL").unwrap_or("".to_string());
    println!("{:?}",connection_string);
    let index_builder = index::index_builder::SqlIndexBuilder{
        connection_string: connection_string
    };  
    let res = index_builder.build_index().await;  

    println!("{:?}",res);
    // println!("Lauching Search API");
    // let ip: String = env::var("SEARCH_IP").unwrap_or("127.0.0.1".to_string());
    // let port = env::var("SEARCH_PORT").unwrap_or("8000".to_string());
    // let bind_address = format!("{}:{}", ip, port);

    // println!("Binding to: {}", bind_address);
    // HttpServer::new(|| {
    //     App::new()
    //         .service(api::endpoints::search)
    //         .service(api::endpoints::relational)
    //         .service(api::endpoints::feedback)
    // })
    // .bind(bind_address)?
    // .run()
    // .await;

    Ok(())
}
