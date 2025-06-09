use crate::constants::APP_NAME;
use dirs;
use std::path::PathBuf;

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
        let exe = env::current_exe().unwrap();
        let mut components = exe.components();
        let root = components.next().unwrap();
        PathBuf::from(root.as_os_str())
    }
}
