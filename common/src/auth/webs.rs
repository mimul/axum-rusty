use http::{header, HeaderMap, HeaderValue};

/// 인증 쿠키 유효 시간 (초)
const COOKIE_MAX_AGE_SECS: i64 = 60;
use log::error;
use tower_cookies::cookie::{time::Duration, CookieBuilder, SameSite};

pub fn create_cookie_headers(key: &str, value: &str) -> Result<header::HeaderMap, String> {
    let cookie = CookieBuilder::new(key, value)
        .path("/")
        .max_age(Duration::seconds(COOKIE_MAX_AGE_SECS))
        //.secure(true) // true: indicates that only https requests will carry
        .http_only(true)
        .same_site(SameSite::Strict)
        .build();
    let header_value = cookie
        .to_string()
        .parse::<HeaderValue>()
        .map_err(|e| format!("쿠키 헤더 파싱 실패: {e}"))?;
    let mut headers = header::HeaderMap::new();
    headers.append(header::SET_COOKIE, header_value);
    Ok(headers)
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
    cookie_str
        .split(';')
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_cookie_from_str_with_single_cookie_returns_value() {
        let result = get_cookie_from_str("access_token=abc123", "access_token");
        assert_eq!(result, Some("abc123".to_string()));
    }

    #[test]
    fn get_cookie_from_str_with_multiple_cookies_returns_correct_value() {
        let result = get_cookie_from_str(
            "session=xyz; access_token=tok789; other=val",
            "access_token",
        );
        assert_eq!(result, Some("tok789".to_string()));
    }

    #[test]
    fn get_cookie_from_str_with_missing_key_returns_none() {
        let result = get_cookie_from_str("session=xyz; other=val", "access_token");
        assert!(result.is_none());
    }

    #[test]
    fn get_cookie_from_str_with_empty_string_returns_none() {
        let result = get_cookie_from_str("", "access_token");
        assert!(result.is_none());
    }

    #[test]
    fn get_auth_header_with_valid_bearer_returns_token() {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::AUTHORIZATION,
            "Bearer mytoken123".parse().unwrap(),
        );
        assert_eq!(get_auth_header(&headers), Some("mytoken123"));
    }

    #[test]
    fn get_auth_header_with_missing_header_returns_none() {
        let headers = HeaderMap::new();
        assert!(get_auth_header(&headers).is_none());
    }

    #[test]
    fn get_auth_header_with_non_bearer_scheme_returns_none() {
        let mut headers = HeaderMap::new();
        headers.insert(
            http::header::AUTHORIZATION,
            "Basic dXNlcjpwYXNz".parse().unwrap(),
        );
        assert!(get_auth_header(&headers).is_none());
    }

    #[test]
    fn get_cookie_from_headers_with_valid_cookie_returns_value() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            "access_token=headertoken; other=val".parse().unwrap(),
        );
        assert_eq!(
            get_cookie_from_headers("access_token", &headers),
            Some("headertoken".to_string())
        );
    }

    #[test]
    fn get_cookie_from_headers_with_no_cookie_header_returns_none() {
        let headers = HeaderMap::new();
        assert!(get_cookie_from_headers("access_token", &headers).is_none());
    }

    #[test]
    fn create_cookie_headers_contains_set_cookie_header() {
        let headers = create_cookie_headers("access_token", "mytoken").unwrap();
        assert!(headers.contains_key(header::SET_COOKIE));
        let val = headers.get(header::SET_COOKIE).unwrap().to_str().unwrap();
        assert!(val.contains("access_token=mytoken"));
        assert!(val.contains("HttpOnly"));
        assert!(val.contains("SameSite=Strict"));
    }

    #[test]
    fn create_cookie_headers_with_valid_input_returns_ok() {
        let headers = create_cookie_headers("token", "abc123")
            .expect("유효한 입력으로 헤더 생성 성공해야 함");
        assert!(headers.contains_key(header::SET_COOKIE));
    }

    #[test]
    fn get_cookie_from_headers_with_key_not_in_cookie_returns_none() {
        let mut headers = HeaderMap::new();
        headers.insert(header::COOKIE, "session=xyz; other=val".parse().unwrap());
        assert!(get_cookie_from_headers("access_token", &headers).is_none());
    }

    #[test]
    fn get_cookie_from_headers_with_no_value_returns_empty_string() {
        let mut headers = HeaderMap::new();
        headers.insert(header::COOKIE, "access_token=".parse().unwrap());
        assert_eq!(
            get_cookie_from_headers("access_token", &headers),
            Some("".to_string())
        );
    }

    #[test]
    fn get_cookie_from_headers_with_key_only_no_equals_returns_none() {
        let mut headers = HeaderMap::new();
        headers.insert(header::COOKIE, "access_token".parse().unwrap());
        let result = get_cookie_from_headers("access_token", &headers);
        assert_eq!(result, Some("".to_string()));
    }
}
