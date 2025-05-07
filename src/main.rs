mod controllers;
use actix_cors::Cors;
use actix_web::{http, web, App, HttpServer};
use controllers::link_controller::create_link;
use controllers::link_controller::delete_link;
use controllers::link_controller::get_link;
use dotenv::dotenv;
use std::env;
use tokio_postgres::NoTls;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let server_config = (env::var("SERVER_HOST")?, env::var("SERVER_PORT")?);

    let db_config = (
        env::var("DB_HOST")?,
        env::var("DB_PORT")?,
        env::var("DB_USER")?,
        env::var("DB_PASSWORD")?,
        env::var("DB_NAME")?,
    );

    let _db_connection_string = format!(
        "host={} port={} user={} password={} dbname={}",
        db_config.0, db_config.1, db_config.2, db_config.3, db_config.4
    );

    let (client, connection) = tokio_postgres::connect(&_db_connection_string, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    client
        .execute(
            "CREATE TABLE IF NOT EXISTS links (
            id VARCHAR(6) PRIMARY KEY,
            original_url TEXT NOT NULL
        );",
            &[],
        )
        .await?;

    let client_data = web::Data::new(client);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("https://localhost:3000")
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
            .app_data(client_data.clone())
            .route("/create", web::post().to(create_link))
            .route("/{id}", web::get().to(get_link))
            .route("/delete/{id}", web::delete().to(delete_link))
    })
    .bind(format!("{}:{}", server_config.0, server_config.1))?
    .run()
    .await?;

    Ok(())
}
