use crate::constants::APP_NAME;
use dirs;
use std::path::PathBuf;
mod model;
pub use model::ConfigModel;

const CONFIG_FILE_NAME: &str = "config.json";

pub fn load_config() -> ConfigModel {
    let config_path = config_dir().join(CONFIG_FILE_NAME);
    if config_path.exists() {
        let config_data = std::fs::read_to_string(&config_path)
            .unwrap_or_else(|_| panic!("failed to read config file: {:?}", config_path));
        serde_json::from_str(&config_data)
            .unwrap_or_else(|_| panic!("failed to parse config file: {:?}", config_path))
    } else {
        ConfigModel::default()
    }
}

pub async fn load_config_async() -> ConfigModel {
    let config_path = config_dir().join(CONFIG_FILE_NAME);
    if config_path.exists() {
        let config_data = tokio::fs::read_to_string(&config_path)
            .await
            .unwrap_or_else(|_| panic!("failed to read config file: {:?}", config_path));
        serde_json::from_str(&config_data)
            .unwrap_or_else(|_| panic!("failed to parse config file: {:?}", config_path))
    } else {
        ConfigModel::default()
    }
}

pub fn save_config(config: &ConfigModel) {
    let config_path = config_dir().join(CONFIG_FILE_NAME);
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .unwrap_or_else(|_| panic!("failed to create config directory: {:?}", parent));
    }
    let config_data = serde_json::to_string_pretty(config)
        .unwrap_or_else(|_| panic!("failed to serialize config: {:?}", config));
    std::fs::write(&config_path, config_data)
        .unwrap_or_else(|_| panic!("failed to write config file: {:?}", config_path));
}

pub async fn save_config_async(config: &ConfigModel) {
    let config_path = config_dir().join(CONFIG_FILE_NAME);
    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .unwrap_or_else(|_| panic!("failed to create config directory: {:?}", parent));
    }
    let config_data = serde_json::to_string_pretty(config)
        .unwrap_or_else(|_| panic!("failed to serialize config: {:?}", config));
    tokio::fs::write(&config_path, config_data)
        .await
        .unwrap_or_else(|_| panic!("failed to write config file: {:?}", config_path));
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
