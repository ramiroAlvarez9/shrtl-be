use actix_web::{web, HttpRequest, HttpResponse, Responder};
use tokio_postgres::Client;

use crate::utils::links::has_valid_api_key;

pub async fn delete_link(
    req: HttpRequest,
    path: web::Path<String>,
    db_client: web::Data<Client>,
    api_key: web::Data<String>,
) -> impl Responder {
    if !has_valid_api_key(&req, api_key.get_ref()) {
        return HttpResponse::Unauthorized().json("Missing or invalid API key");
    }

    let id = path.into_inner();
    match db_client
        .execute("DELETE FROM links WHERE id = $1;", &[&id])
        .await
    {
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
