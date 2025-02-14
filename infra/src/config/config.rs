use std::env;
use tracing::info;

#[derive(Debug, Clone)]
pub struct ApplicationConfig {
    pub debug: bool,
    pub database_url: String,
    pub jwt_secret: String,
    pub allowed_origin: String,
    pub jwt_duration: String,
    pub jwt_max_age: i64,
    pub log_dir: String,
    pub log_rolling: String,
    pub log_pack_compress: String,
    pub log_keep_type: String,
    pub log_level: String,
    pub log_chan_len: Option<usize>
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
        let log_dir = env::var("LOG_DIR").expect("LOG_DIR must be set!");
        let log_rolling = env::var("LOG_ROLLING").unwrap_or_else(|_| panic!("LOG_ROLLING must be set!"));
        let log_pack_compress = env::var("LOG_PACK_COMPRESS").expect("LOG_PACK_COMPRESS must be set!");
        let log_keep_type = env::var("LOG_KEEP_TYPE").expect("LOG_KEEP_TYPE must be set!");
        let log_level = env::var("LOG_LEVEL").expect("LOG_LEVEL must be set!");
        let log_chan_len = env::var("LOG_CHAN_LEN").expect("LOG_CHAN_LEN must be set!");

        ApplicationConfig {
            debug: debug.parse::<bool>().unwrap(),
            database_url,
            jwt_secret,
            allowed_origin,
            jwt_duration,
            jwt_max_age: jwt_max_age.parse::<i64>().unwrap(),
            log_dir,
            log_rolling,
            log_pack_compress,
            log_keep_type,
            log_level,
            log_chan_len: Option::from(log_chan_len.parse::<usize>().unwrap())
        }
    }
}