use sqlx::SqlitePool;

pub mod config;
pub mod db;
pub mod routes;

pub use config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Config,
}
