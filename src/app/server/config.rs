use serde::Deserialize;
use std::{net::SocketAddr, path::PathBuf};

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub server: ServerSection,
    pub storage: StorageSection,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServerSection {
    pub addr: SocketAddr,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StorageSection {
    pub media_root: PathBuf,
    pub data_dir: PathBuf,
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = std::env::var("HOMEX_CONFIG").unwrap_or_else(|_| "homex.toml".into());
        let raw = std::fs::read_to_string(&path).map_err(|e| format!("cannot read {path}: {e}"))?;
        Ok(toml::from_str(&raw)?)
    }

    pub fn db_path(&self) -> PathBuf {
        self.storage.data_dir.join("homex.db")
    }
}
