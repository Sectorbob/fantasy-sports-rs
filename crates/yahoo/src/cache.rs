use log::info;
use sha2::{Digest, Sha256};
use std::io::Error;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// The trait for caching
#[async_trait::async_trait]
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
    async fn load_cache(&self, key: &str) -> Option<String> {
        let path = self.cache_file_path(key);
        let mut f = fs::File::open(path).await.ok()?;
        let mut s = String::new();
        f.read_to_string(&mut s).await.ok()?;
        Some(s)
    }

    /// Save a string value to the cache for a key.
    async fn save_cache(&self, key: &str, value: &str) -> Result<(), Error> {
        let path = self.cache_file_path(key);
        info!("caching {key} to {:?}", path);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        if let Ok(mut f) = fs::File::create(path).await {
            f.write_all(value.as_bytes()).await?;
        }
        Ok(())
    }
}

pub struct FileSystemCacher {
    cache_dir: PathBuf,
}
impl FileSystemCacher {
    pub fn new<T: Into<PathBuf>>(cache_dir: T) -> Self {
        FileSystemCacher {
            cache_dir: cache_dir.into(),
        }
    }
}
impl Cacheable for FileSystemCacher {
    fn cache_dir(&self) -> std::path::PathBuf {
        self.cache_dir.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::cache::{Cacheable, FileSystemCacher};

    #[test]
    fn test_hash_url() {
        let url = "https://fantasysports.yahooapis.com/fantasy/v2/league/359.l.564503/transactions";
        let expected =
            "cache/yahoo/85eb03b7afd37daecfbdfb16ba8e6751b180ab2d736d246d4184cf53d1ee9385";
        // Setup Fixture
        let fixture = FileSystemCacher::new(Path::new("cache/yahoo"));

        assert_eq!(Some(expected), fixture.cache_file_path(url).to_str());
    }
}
