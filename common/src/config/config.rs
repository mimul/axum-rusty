use std::env;
use log::info;

#[derive(Debug, Clone)]
pub struct ApplicationConfig {
    pub debug: bool,
    pub database_url: String,
    pub jwt_secret: String,
    pub allowed_origin: String,
    pub jwt_duration: String,
    pub jwt_max_age: i64
}

impl ApplicationConfig {
    pub fn init() -> ApplicationConfig {
        info!("ApplicationConfig::init() called.");
        let debug = env::var("DEBUG").expect("DEBUG must be set!");
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set!");
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| panic!("JWT_SECRET must be set!"));
        let allowed_origin = env::var("ALLOWED_ORIGIN").unwrap_or_else(|_| panic!("ALLOWED_ORIGIN must be set!"));
        let jwt_duration = env::var("JWT_DURATION_MINUTES").unwrap_or_else(|_| panic!("JWT_DURATION_MINUTES must be set!"));
        let jwt_max_age = env::var("JWT_MAX_AGE").expect("JWT_MAX_AGE must be set");

        ApplicationConfig {
            debug: debug.parse::<bool>().unwrap(),
            database_url,
            jwt_secret,
            allowed_origin,
            jwt_duration,
            jwt_max_age: jwt_max_age.parse::<i64>().unwrap(),
        }
    }
}