use std::sync::Arc;

use redis::aio::MultiplexedConnection;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tracing::info;

use migration::{Migrator, MigratorTrait};

use crate::config::settings::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub redis: MultiplexedConnection,
    pub config: Arc<Config>,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing application state...");

        // Build database URL and connect
        let db_url = config.database_url();
        info!(database_url = %db_url.split('@').next().unwrap_or("hidden"), "Connecting to database");

        let mut opt = ConnectOptions::new(db_url);
        opt.max_connections(config.db_pool_size);

        let db = Database::connect(opt).await?;
        info!("Database connection established");

        // Run migrations
        info!("Running database migrations...");
        Migrator::up(&db, None).await?;
        info!("Migrations completed");

        // Connect to Redis
        let redis_url = config.redis_url();
        info!(redis_url = %redis_url.split('@').last().unwrap_or("hidden"), "Connecting to Redis");

        let redis_client = redis::Client::open(redis_url)?;
        let redis = redis_client.get_multiplexed_tokio_connection().await?;
        info!("Redis connection established");

        Ok(Self {
            db,
            redis,
            config: Arc::new(config),
        })
    }
}
