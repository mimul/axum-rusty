use http::{header, HeaderMap, HeaderValue};
use tower_cookies::cookie::{ time::Duration, CookieBuilder, SameSite};
use tracing::log::error;

pub fn create_cookie_headers(key: &str, value: &str) -> header::HeaderMap {
    let cookie = CookieBuilder::new(key, value)
        .path("/")
        .max_age(Duration::seconds(60))
        //.secure(true) // true: indicates that only https requests will carry
        .http_only(true)
        .same_site(SameSite::Strict)
        .build();
    let header_value = cookie.to_string().parse::<HeaderValue>().expect("Failed to parse cookie");
    let mut headers = header::HeaderMap::new();
    headers.append(header::SET_COOKIE, header_value); // Will cover!
    headers
}

pub fn get_cookie_from_headers(key: &str, headers: &HeaderMap) -> Option<String> {
    headers.get(header::COOKIE).and_then(|cookie_header| {
        cookie_header
            .to_str()
            .ok()
            .and_then(|cookie_str| get_cookie_from_str(cookie_str, key))
    })
}

pub fn get_cookie_from_str(cookie_str: &str, key: &str) -> Option<String> {
    cookie_str.split(';')
        .map(|pair| {
            let mut parts = pair.trim().splitn(2, '=');
            let name = parts.next().unwrap_or("").to_string();
            let value = parts.next().unwrap_or("").to_string();
            (name, value)
        })
        .find(|(name, _)| name == key)
        .map(|(_, value)| value)
}

pub fn get_auth_header(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            if header.starts_with("Bearer ") {
                header.strip_prefix("Bearer ")
            } else {
                error!("auth_header not found");
                None
            }
        })
}