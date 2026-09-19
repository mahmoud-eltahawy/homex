use sqlx::SqlitePool;
use std::sync::Arc;

pub mod config;
pub mod db;
pub mod routes;

pub use config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Arc<Config>,
}
