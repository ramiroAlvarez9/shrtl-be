use actix_web::HttpRequest;
use url::Url;
use uuid::Uuid;

pub fn has_valid_api_key(req: &HttpRequest, expected_key: &str) -> bool {
    req.headers()
        .get("x-api-key")
        .and_then(|header| header.to_str().ok())
        .map(|provided| provided == expected_key)
        .unwrap_or(false)
}

pub fn normalize_https_url(raw_url: &str) -> Result<String, &'static str> {
    let trimmed = raw_url.trim();
    if trimmed.is_empty() {
        return Err("Invalid URL format");
    }

    let parsed = match Url::parse(trimmed) {
        Ok(url) => url,
        Err(_) => {
            let candidate = format!("https://{}", trimmed);
            Url::parse(&candidate).map_err(|_| "Invalid URL format")?
        }
    };

    if parsed.scheme() != "https" {
        return Err("Only https URLs are allowed");
    }

    if parsed.host_str().is_none() {
        return Err("Invalid URL format");
    }

    Ok(parsed.to_string())
}

pub fn generate_short_id() -> String {
    let uuid = Uuid::new_v4();
    uuid.to_string()
        .chars()
        .filter(|c| c.is_alphanumeric())
        .take(6)
        .collect::<String>()
}
