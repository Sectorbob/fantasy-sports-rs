use dirs;
use std::path::PathBuf;

const DEFAULT_SUB_DIR: &str = "fantasy";

pub fn config_dir() -> PathBuf {
    match dirs::config_dir() {
        Some(home_dir) => home_dir.join(DEFAULT_SUB_DIR),
        None => get_temp_dir().join(DEFAULT_SUB_DIR).join("config"),
    }
}

pub fn data_dir() -> PathBuf {
    match dirs::data_local_dir() {
        Some(data_dir) => data_dir.join(DEFAULT_SUB_DIR),
        None => get_temp_dir().join(DEFAULT_SUB_DIR).join("data"),
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
