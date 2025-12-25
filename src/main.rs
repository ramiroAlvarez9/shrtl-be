mod controllers;
mod utils;
use actix_cors::Cors;
use actix_web::{http, web, App, HttpServer};
use controllers::create_link;
use controllers::delete_link;
use controllers::get_link;
use deadpool_postgres::{Config, Runtime};
use dotenv::dotenv;
use std::env;
use tokio_postgres::NoTls;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let mut cfg = Config::new();
    cfg.host = env::var("DB_HOST").ok();
    cfg.user = env::var("DB_USER").ok();
    cfg.password = env::var("DB_PASSWORD").ok();
    cfg.dbname = env::var("DB_NAME").ok();
    cfg.port = env::var("DB_PORT").ok().and_then(|p| p.parse().ok());

    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)?;

    let api_key = env::var("API_KEY").expect("API_KEY is not set in the environment");

    let pool_data = web::Data::new(pool);
    let api_key_data = web::Data::new(api_key);

    let server_host = env::var("SERVER_HOST")?;
    let server_port = env::var("SERVER_PORT")?;

    println!("Starting server on {}:{}", server_host, server_port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("https://shrtl-peach.vercel.app")
            .allowed_methods(vec!["GET", "POST", "DELETE"])
            .allowed_headers(vec![
                http::header::AUTHORIZATION,
                http::header::ACCEPT,
                http::header::CONTENT_TYPE,
            ])
            .allowed_header("x-api-key")
            .max_age(3600);
        App::new()
            .wrap(cors)
            .app_data(pool_data.clone())
            .app_data(api_key_data.clone())
            .route("/create", web::post().to(create_link))
            .route("/delete/{id}", web::delete().to(delete_link))
            .route("/{id}", web::get().to(get_link))
    })
    .bind(format!("{}:{}", server_host, server_port))?
    .run()
    .await?;

    Ok(())
}
