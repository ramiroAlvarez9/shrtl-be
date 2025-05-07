use actix_web::{web, HttpResponse, Responder};
use regex::Regex;
use serde::Deserialize;
use tokio_postgres::Client;
use uuid::Uuid;

#[derive(Deserialize)]
    pub struct LinkData {
        url: String,
    }
    pub async fn create_link(
        link_data: web::Json<LinkData>,
        db_client: web::Data<Client> 
    ) -> impl Responder {
        let original_link = link_data.url.clone();
        let id = generate_short_id(); 
        if is_valid_url(&original_link) {
            match db_client.execute(
                "INSERT INTO links (id, original_url) VALUES ($1, $2)", 
                &[&id, &original_link]
            ).await {
                Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "id": id })),
                Err(e) => {
                    eprintln!("Database error: {}", e);
                    HttpResponse::InternalServerError().json("Repeated ID")
                }
            }
        } else {
            HttpResponse::BadRequest().json("Invalid URL format")
        }
    }

    pub async fn get_link(
        path: web::Path<String>,
        db_client: web::Data<Client>
    ) -> impl Responder {
        let id = path.into_inner();
        match db_client.query_one(
            "SELECT original_url FROM links WHERE id = ($1);", 
            &[&id]
        ).await {
            Ok(row) => {
                let mut original_url: String = row.get("original_url");
                if !original_url.starts_with("http://") && !original_url.starts_with("https://") {
                    original_url = format!("https://{}", original_url);
                }
                HttpResponse::Found()
                    .append_header(("Location", original_url))
                    .finish()
            }
            Err(e) => {
                eprintln!("Database error: {}", e);
                HttpResponse::InternalServerError().json("Failed to recover URL")
            }
        }
    }

    pub async fn delete_link(
        path: web::Path<String>,
        db_client: web::Data<Client> 
    ) -> impl Responder {
        let id = path.into_inner();
        match db_client.execute(
            "DELETE FROM links WHERE id = $1;",
            &[&id]
        ).await {
            Ok(rows_deleted) => {
                if rows_deleted > 0 {
                    HttpResponse::Ok().json("Link deleted successfully")
                } else {
                    HttpResponse::NotFound().json("Link not found")
                }
            }
            Err(e) => {
                eprintln!("Database error: {}", e);
                HttpResponse::InternalServerError().json("Failed to delete URL")
            }
        }
    }

    fn is_valid_url(url: &str) -> bool {
        let url_regex = Regex::new(r"^(https?://)?([a-zA-Z0-9-]+\.)+[a-zA-Z]{2,}(/[a-zA-Z0-9-._~:/?#\[\]@!$&'()*+,;=]*)?$").unwrap();
        url_regex.is_match(url)
    }

    fn generate_short_id() -> String {
        let uuid = Uuid::new_v4();
        uuid.to_string()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .take(6)
            .collect::<String>()
    }

