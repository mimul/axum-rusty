use log::info;
use std::env;
use std::fmt;

// ---------------------------------------------------------------------------
// ConfigError
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum ConfigError {
    MissingEnvVar(&'static str),
    ParseError(&'static str, String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::MissingEnvVar(name) => {
                write!(f, "환경변수 {name} 이(가) 설정되지 않았습니다")
            }
            ConfigError::ParseError(name, reason) => {
                write!(f, "환경변수 {name} 파싱 실패: {reason}")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

// ---------------------------------------------------------------------------
// ApplicationConfig
// ---------------------------------------------------------------------------

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
    /// 환경변수에서 설정을 읽는다.
    /// 누락·파싱 오류 시 panic 대신 `ConfigError`를 반환한다.
    pub fn try_init() -> Result<ApplicationConfig, ConfigError> {
        info!("ApplicationConfig::try_init() called.");

        let debug = require_env("DEBUG")?
            .parse::<bool>()
            .map_err(|e| ConfigError::ParseError("DEBUG", e.to_string()))?;

        let database_url = require_env("DATABASE_URL")?;
        let jwt_secret = require_env("JWT_SECRET")?;
        let allowed_origin = require_env("ALLOWED_ORIGIN")?;

        let jwt_duration = require_env("JWT_DURATION_MINUTES")?
            .parse::<i64>()
            .map_err(|e| ConfigError::ParseError("JWT_DURATION_MINUTES", e.to_string()))?;

        let jwt_max_age = require_env("JWT_MAX_AGE")?
            .parse::<i64>()
            .map_err(|e| ConfigError::ParseError("JWT_MAX_AGE", e.to_string()))?;

        Ok(ApplicationConfig {
            debug,
            database_url,
            jwt_secret,
            allowed_origin,
            jwt_duration,
            jwt_max_age,
        })
    }
}

fn require_env(name: &'static str) -> Result<String, ConfigError> {
    env::var(name).map_err(|_| ConfigError::MissingEnvVar(name))
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
        let config = ApplicationConfig::try_init().expect("설정 파싱 성공해야 함");
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
        let config = ApplicationConfig::try_init().expect("설정 파싱 성공해야 함");
        assert!(config.debug);
    }

    #[test]
    fn application_config_jwt_max_age_parses_as_i64() {
        set_env_vars("false");
        env::set_var("JWT_MAX_AGE", "7200");
        let config = ApplicationConfig::try_init().expect("설정 파싱 성공해야 함");
        assert_eq!(config.jwt_max_age, 7200i64);
    }

    #[test]
    fn application_config_returns_error_when_env_var_missing() {
        set_env_vars("false");
        env::remove_var("JWT_SECRET");
        let result = ApplicationConfig::try_init();
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("JWT_SECRET"));
    }

    #[test]
    fn application_config_returns_error_when_debug_is_invalid() {
        set_env_vars("not-a-bool");
        let result = ApplicationConfig::try_init();
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("DEBUG"));
    }
}
