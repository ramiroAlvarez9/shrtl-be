use actix_web::{web, HttpRequest, HttpResponse, Responder};
use deadpool_postgres::Pool;
use serde::Deserialize;

use crate::utils::links::{generate_short_id, has_valid_api_key, normalize_https_url};

#[derive(Deserialize)]
pub struct LinkData {
    url: String,
}

pub async fn create_link(
    req: HttpRequest,
    link_data: web::Json<LinkData>,
    pool: web::Data<Pool>,
    api_key: web::Data<String>,
) -> impl Responder {
    if !has_valid_api_key(&req, api_key.get_ref()) {
        return HttpResponse::Unauthorized().json("Missing or invalid API key");
    }

    let id = generate_short_id();
    let normalized_url = match normalize_https_url(&link_data.url) {
        Ok(url) => url,
        Err(message) => return HttpResponse::BadRequest().json(message),
    };

    let client = match pool.get().await {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Error getting client from pool: {}", e);
            return HttpResponse::InternalServerError().json("Failed to connect to database");
        }
    };

    match client
        .execute(
            "INSERT INTO links (id, original_url) VALUES ($1, $2)",
            &[&id, &normalized_url],
        )
        .await
    {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "id": id })),
        Err(e) => {
            eprintln!("Database error: {}", e);
            HttpResponse::InternalServerError().json("Repeated ID")
        }
    }
}
