use log::info;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

// The trait for cachinh
pub trait Cacheable {
    /// Returns the cache directory location, or None if caching is off.
    fn cache_dir(&self) -> PathBuf;

    /// Given a cache key, build a filesystem-safe path in the cache directory.
    fn cache_file_path(&self, key: &str) -> PathBuf {
        let dir = self.cache_dir();
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        dir.join(hash)
    }

    /// Try to load a cached string value for a key.
    fn load_cache(&self, key: &str) -> Option<String> {
        let path = self.cache_file_path(key);
        let mut f = fs::File::open(path).ok()?;
        let mut s = String::new();
        f.read_to_string(&mut s).ok()?;
        Some(s)
    }

    /// Save a string value to the cache for a key.
    fn save_cache(&self, key: &str, value: &str) {
        let path = self.cache_file_path(key);
        info!("caching {key} to {:?}", path);
    
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(mut f) = fs::File::create(path) {
            let _ = f.write_all(value.as_bytes());
        }
    }
}

struct CacheableFoo {}

impl Cacheable for CacheableFoo {
    fn cache_dir(&self) -> PathBuf {
        todo!()
    }
}
// impl Sync for CacheableFoo {

// }
