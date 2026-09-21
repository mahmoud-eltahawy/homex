use leptos::prelude::ServerFnError;
use sqlx::SqlitePool;

pub mod config;
pub mod convert;
pub mod db;
pub mod poster;
pub mod routes;
pub mod upload;

pub use config::Config;
pub use convert::Jobs;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Config,
    pub jobs: Jobs,
}

pub trait SqlErr<T> {
    fn srv(self) -> Result<T, ServerFnError>;
}

impl<T> SqlErr<T> for Result<T, sqlx::Error> {
    fn srv(self) -> Result<T, ServerFnError> {
        self.map_err(|e| ServerFnError::new(e.to_string()))
    }
}
