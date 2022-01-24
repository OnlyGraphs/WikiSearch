mod api;
use actix_web::{HttpServer,App};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Lauching Search API");

    let ip: String = 
        match env::var("SEARCH_IP"){
            Ok(x) => x, 
            Err(_e) => "127.0.0.1".to_owned(),
    };

    let port = 
        match env::var("SEARCH_PORT"){
            Ok(x) => x,
            Err(_e) => "80".to_owned(),
    };

    let bind_address = format!("{}:{}",ip,port);

    println!("Binding to: {}",bind_address);
    HttpServer::new(|| {
        App::new()
            .service(api::endpoints::search)
            .service(api::endpoints::relational)
    })

    .bind(bind_address)?
    .run()
    .await
    
}