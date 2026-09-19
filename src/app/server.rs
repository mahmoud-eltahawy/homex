use sqlx::SqlitePool;

pub mod config;
pub mod convert;
pub mod db;
pub mod routes;

pub use config::Config;
pub use convert::Jobs;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Config,
    pub jobs: Jobs,
}
