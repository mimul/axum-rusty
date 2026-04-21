use log::info;
use std::env;

#[derive(Debug, Clone)]
pub struct ApplicationConfig {
    pub debug: bool,
    pub database_url: String,
    pub jwt_secret: String,
    pub allowed_origin: String,
    pub jwt_duration: i64,
    pub jwt_max_age: i64,
}

impl ApplicationConfig {
    pub fn init() -> ApplicationConfig {
        info!("ApplicationConfig::init() called.");
        let debug = env::var("DEBUG").expect("DEBUG must be set!");
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set!");
        let jwt_secret =
            env::var("JWT_SECRET").unwrap_or_else(|_| panic!("JWT_SECRET must be set!"));
        let allowed_origin =
            env::var("ALLOWED_ORIGIN").unwrap_or_else(|_| panic!("ALLOWED_ORIGIN must be set!"));
        let jwt_duration = env::var("JWT_DURATION_MINUTES")
            .unwrap_or_else(|_| panic!("JWT_DURATION_MINUTES must be set!"))
            .parse::<i64>()
            .expect("JWT_DURATION_MINUTES must be an integer");
        let jwt_max_age = env::var("JWT_MAX_AGE").expect("JWT_MAX_AGE must be set");

        ApplicationConfig {
            debug: debug
                .parse::<bool>()
                .expect("DEBUG must be a boolean (true/false)"),
            database_url,
            jwt_secret,
            allowed_origin,
            jwt_duration,
            jwt_max_age: jwt_max_age
                .parse::<i64>()
                .expect("JWT_MAX_AGE must be an integer"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn set_env_vars(debug: &str) {
        env::set_var("DEBUG", debug);
        env::set_var("DATABASE_URL", "postgres://localhost/testdb");
        env::set_var("JWT_SECRET", "test-jwt-secret");
        env::set_var("ALLOWED_ORIGIN", "http://localhost:3000");
        env::set_var("JWT_DURATION_MINUTES", "60");
        env::set_var("JWT_MAX_AGE", "3600");
    }

    #[test]
    fn application_config_init_reads_all_env_vars() {
        set_env_vars("false");
        let config = ApplicationConfig::init();
        assert!(!config.debug);
        assert_eq!(config.database_url, "postgres://localhost/testdb");
        assert_eq!(config.jwt_secret, "test-jwt-secret");
        assert_eq!(config.allowed_origin, "http://localhost:3000");
        assert_eq!(config.jwt_duration, 60i64);
        assert_eq!(config.jwt_max_age, 3600);
    }

    #[test]
    fn application_config_debug_true_parses_correctly() {
        set_env_vars("true");
        let config = ApplicationConfig::init();
        assert!(config.debug);
    }

    #[test]
    fn application_config_jwt_max_age_parses_as_i64() {
        set_env_vars("false");
        env::set_var("JWT_MAX_AGE", "7200");
        let config = ApplicationConfig::init();
        assert_eq!(config.jwt_max_age, 7200i64);
    }
}
