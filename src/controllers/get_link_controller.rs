use actix_web::{web, HttpResponse, Responder};
use deadpool_postgres::Pool;

use crate::utils::links::normalize_https_url;

pub async fn get_link(path: web::Path<String>, pool: web::Data<Pool>) -> impl Responder {
    let id = path.into_inner();
    let client = match pool.get().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Error getting client from pool: {}", e);
            return HttpResponse::InternalServerError().json("Failed to connect to database");
        }
    };
    match client
        .query_one("SELECT original_url FROM links WHERE id = ($1);", &[&id])
        .await
    {
        Ok(row) => {
            let original_url: String = row.get("original_url");
            let redirect_url = match normalize_https_url(&original_url) {
                Ok(url) => url,
                Err(_) => return HttpResponse::InternalServerError().json("Failed to recover URL"),
            };
            HttpResponse::Found()
                .append_header(("Location", redirect_url))
                .finish()
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().json("Failed to recover URL")
        }
    }
}
