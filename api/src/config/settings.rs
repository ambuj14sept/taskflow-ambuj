use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    // Database
    pub db_host: String,
    pub db_port: u16,
    pub db_user: String,
    pub db_password: String,
    pub db_name: String,
    pub db_pool_size: u32,

    // Redis
    pub redis_host: String,
    pub redis_port: u16,
    pub redis_password: Option<String>,
    pub redis_db: u8,

    // Auth
    pub jwt_secret: String,
    pub jwt_expiry_hours: u64,
    pub bcrypt_cost: u32,

    // Server
    pub server_host: String,
    pub server_port: u16,

    // Logging
    pub env: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            // Database
            db_host: env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string()),
            db_port: env::var("DB_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .expect("DB_PORT must be a valid u16"),
            db_user: env::var("DB_USER").unwrap_or_else(|_| "taskflow".to_string()),
            db_password: env::var("DB_PASSWORD").unwrap_or_else(|_| "taskflow_secret".to_string()),
            db_name: env::var("DB_NAME").unwrap_or_else(|_| "taskflow".to_string()),
            db_pool_size: env::var("DB_POOL_SIZE")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .expect("DB_POOL_SIZE must be a valid u32"),

            // Redis
            redis_host: env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".to_string()),
            redis_port: env::var("REDIS_PORT")
                .unwrap_or_else(|_| "6379".to_string())
                .parse()
                .expect("REDIS_PORT must be a valid u16"),
            redis_password: env::var("REDIS_PASSWORD").ok().filter(|s| !s.is_empty()),
            redis_db: env::var("REDIS_DB")
                .unwrap_or_else(|_| "0".to_string())
                .parse()
                .expect("REDIS_DB must be a valid u8"),

            // Auth
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            jwt_expiry_hours: env::var("JWT_EXPIRY_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .expect("JWT_EXPIRY_HOURS must be a valid u64"),
            bcrypt_cost: env::var("BCRYPT_COST")
                .unwrap_or_else(|_| "12".to_string())
                .parse()
                .expect("BCRYPT_COST must be a valid u32"),

            // Server
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "9090".to_string())
                .parse()
                .expect("SERVER_PORT must be a valid u16"),

            // Logging
            env: env::var("ENV").unwrap_or_else(|_| "dev".to_string()),
        }
    }

    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.db_user, self.db_password, self.db_host, self.db_port, self.db_name
        )
    }

    pub fn redis_url(&self) -> String {
        match &self.redis_password {
            Some(password) => {
                format!(
                    "redis://:{}@{}:{}/{}",
                    password, self.redis_host, self.redis_port, self.redis_db
                )
            }
            None => {
                format!(
                    "redis://{}:{}/{}",
                    self.redis_host, self.redis_port, self.redis_db
                )
            }
        }
    }
}
