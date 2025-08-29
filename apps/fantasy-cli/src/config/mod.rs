use crate::constants::APP_NAME;
use dirs;
use std::{io, path::PathBuf};
mod model;
pub use model::*;

const CONFIG_FILE_NAME: &str = "config.json";

pub fn load_config() -> Result<ConfigModel> {
    let config_path = config_dir().join(CONFIG_FILE_NAME);
    if config_path.exists() {
        let config_data = std::fs::read_to_string(&config_path)?;
        Ok(serde_json::from_str(&config_data)?)
    } else {
        Ok(ConfigModel::default())
    }
}

pub async fn load_config_async() -> Result<ConfigModel> {
    let config_path = config_dir().join(CONFIG_FILE_NAME);
    if config_path.exists() {
        let config_data = tokio::fs::read_to_string(&config_path).await?;
        Ok(serde_json::from_str(&config_data)?)
    } else {
        let cfg = ConfigModel::default();
        save_config_async(&cfg).await?;
        Ok(cfg)
    }
}

pub fn save_config(config: &ConfigModel) -> Result<()> {
    let config_path = config_dir().join(CONFIG_FILE_NAME);
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .unwrap_or_else(|_| panic!("failed to create config directory: {:?}", parent));
    }
    let config_data = serde_json::to_string_pretty(config)?;
    std::fs::write(&config_path, config_data)
}

pub async fn save_config_async(config: &ConfigModel) -> Result<()> {
    let config_path = config_dir().join(CONFIG_FILE_NAME);
    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let config_data = serde_json::to_string_pretty(config)?;
    tokio::fs::write(&config_path, config_data).await
}

pub fn cache_dir() -> PathBuf {
    match dirs::cache_dir() {
        Some(data_dir) => data_dir.join(APP_NAME),
        None => get_temp_dir().join(APP_NAME).join("cache"),
    }
}

pub fn config_dir() -> PathBuf {
    match dirs::config_dir() {
        Some(home_dir) => home_dir.join(APP_NAME),
        None => get_temp_dir().join(APP_NAME).join("config"),
    }
}

pub fn data_dir() -> PathBuf {
    match dirs::data_local_dir() {
        Some(data_dir) => data_dir.join(APP_NAME),
        None => get_temp_dir().join(APP_NAME).join("data"),
    }
}

fn get_temp_dir() -> PathBuf {
    #[cfg(unix)]
    {
        PathBuf::from("/tmp")
    }
    #[cfg(windows)]
    {
        let exe = env::current_exe().expect("failed to get current exe");
        let mut components = exe.components();
        let root = components.next().unwrap();
        PathBuf::from(root.as_os_str())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("io error")]
    IoError(#[from] io::Error),
    #[error("io error")]
    SerdeError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, io::Error>;
