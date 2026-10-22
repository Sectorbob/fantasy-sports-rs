use crate::{Cacheable, Entry};
use chrono::Utc;
use sha2::Digest;
use sha2::Sha256;
use std::{
    collections::HashMap,
    io::{Read, Seek},
    path::PathBuf,
    sync::Arc,
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
    sync::Mutex,
};

const DEFAULT_TTL: u64 = 24 * 60 * 60; // 24 hours
const METADATA_FILE_NAME: &str = "metadata.mpk";

pub struct FileSystemCacher {
    metadata: Arc<Mutex<(File, HashMap<String, Entry>)>>,
    cache_dir: PathBuf,
}
impl FileSystemCacher {
    /// Returns the cache directory location, or None if caching is off.
    fn cache_dir(&self) -> std::path::PathBuf {
        self.cache_dir.clone()
    }

    /// Given a cache key, build a filesystem-safe path in the cache directory.
    fn cache_file_path(&self, hash: &str) -> PathBuf {
        let dir = self.cache_dir();
        dir.join(hash)
    }

    pub fn open_sync<T: Into<PathBuf>>(cache_dir: T) -> Result<Self, std::io::Error> {
        let cache_dir = cache_dir.into();

        let f = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(cache_dir.join(METADATA_FILE_NAME))?;
        // if let Err(try_lock_error) = f.into_std().await.try_lock() {
        //     panic!("unable to lock cache directory: {try_lock_error:?}")
        // }

        let file_has_data = f.metadata()?.len() > 0;

        let cache = FileSystemCacher {
            metadata: Arc::new(Mutex::new((f.into(), HashMap::new()))), // TODO: load from file
            cache_dir,
        };
        if file_has_data {
            cache.load_metadata_file_sync()?;
        }
        Ok(cache)
    }

    pub async fn open<T: Into<PathBuf>>(cache_dir: T) -> Result<Self, tokio::io::Error> {
        let cache_dir = cache_dir.into();

        let f = tokio::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(cache_dir.join(METADATA_FILE_NAME))
            .await?;
        // if let Err(try_lock_error) = f.into_std().await.try_lock() {
        //     panic!("unable to lock cache directory: {try_lock_error:?}")
        // }

        let file_has_data = f.metadata().await?.len() > 0;

        let cache = FileSystemCacher {
            metadata: Arc::new(Mutex::new((f, HashMap::new()))), // TODO: load from file
            cache_dir,
        };
        if file_has_data {
            cache.load_metadata_file().await?;
        }

        Ok(cache)
    }

    fn load_metadata_file_sync(&self) -> Result<(), std::io::Error> {
        let mut metadata_guard = self.metadata.blocking_lock();
        let mut tokio_file = tokio::fs::File::from_std(std::fs::File::create("/dev/null")?);
        std::mem::swap(&mut tokio_file, &mut metadata_guard.0); // this is gross
        let mut buf = vec![];
        let mut std_file = tokio_file.try_into_std().map_err(|file| {
            metadata_guard.0 = file;
            std::io::Error::other("unable to convert tokio file to std file")
        })?;
        if let Err(err) = std_file.read_to_end(&mut buf) {
            metadata_guard.0 = tokio::fs::File::from_std(std_file); // set it back
            return Err(err);
        }
        if let Err(err) = std_file.seek(std::io::SeekFrom::Start(0)) {
            metadata_guard.0 = tokio::fs::File::from_std(std_file); // set it back
            return Err(err);
        }
        metadata_guard.0 = tokio::fs::File::from_std(std_file);
        metadata_guard.1 = rmp_serde::from_slice(&buf).map_err(to_io_err)?;
        Ok(())
    }

    async fn load_metadata_file(&self) -> Result<(), std::io::Error> {
        let mut buf = vec![];
        let mut metadata_guard = self.metadata.lock().await;
        metadata_guard.0.read_to_end(&mut buf).await?;
        metadata_guard.1 = rmp_serde::from_slice(&buf).map_err(to_io_err)?;
        metadata_guard.0.seek(std::io::SeekFrom::Start(0)).await?; // move file cursor back to start
        Ok(())
    }
}

#[async_trait::async_trait]
impl Cacheable for FileSystemCacher {
    async fn get_metadata(&self, k: String) -> Result<Option<Entry>, std::io::Error> {
        Ok(self.metadata.lock().await.1.get(&k).cloned())
    }

    async fn save_metadata(&self, k: String, v: Entry) -> Result<(), std::io::Error> {
        let mut metadata_guard = self.metadata.lock().await;
        metadata_guard.1.insert(k, v);
        let data = rmp_serde::to_vec(&metadata_guard.1).map_err(to_io_err)?;
        metadata_guard.0.write_all(&data).await?;
        metadata_guard.0.seek(std::io::SeekFrom::Start(0)).await?; // move file cursor back to start
        metadata_guard.0.flush().await?;
        Ok(())
    }

    /// Try to load a cached string value for a key.
    async fn load_cache(&self, key: &str) -> Option<Vec<u8>> {
        let hash = hash(key);

        // check ttl
        if let Some(meta_entry) = self.metadata.lock().await.1.get(&hash)
            && !meta_entry.is_expired()
        {
            let path = self.cache_file_path(hash.as_str());
            let mut f = tokio::fs::File::open(path).await.ok()?;
            let mut buf = Vec::new();
            f.read_to_end(&mut buf).await.ok()?;
            return Some(buf);
        }
        None
    }

    /// Save a string value to the cache for a key.
    async fn save_cache<'a>(&self, key: &str, value: &'a [u8]) -> Result<(), std::io::Error> {
        let hash = hash(key);
        let path = self.cache_file_path(hash.as_str());
        log::debug!("caching {key} to {path:?}");

        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        if let Ok(mut f) = tokio::fs::File::create(path).await {
            f.write_all(value).await?;
        }
        self.save_metadata(
            hash,
            Entry {
                ts: Utc::now(),
                ttl: DEFAULT_TTL,
            },
        )
        .await?;
        Ok(())
    }
}

fn hash(key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn to_io_err<T>(err: T) -> std::io::Error
where
    T: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    std::io::Error::new(std::io::ErrorKind::InvalidData, err)
}

#[cfg(test)]
mod fs_cacher_tests {
    use crate::{Cacheable, FileSystemCacher, cache::fs_cache::METADATA_FILE_NAME};
    use base64::Engine;
    use chrono::Utc;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_open_cache_async() {
        let tmp_dir = TempDir::new().expect("failed to create temp directory");
        let dir_path = tmp_dir.path();
        debug_assert!(
            !dir_path.join("metadata.mpk").exists(),
            "metadata file should not exist yet"
        );
        let fixture = FileSystemCacher::open(dir_path)
            .await
            .expect("failed to open cache");
        debug_assert_eq!(dir_path, &fixture.cache_dir, "cache dir is expected");
        debug_assert!(
            dir_path.join("metadata.mpk").exists(),
            "metadata file should exist now"
        );
        let metadata = fixture
            .metadata
            .lock()
            .await
            .0
            .metadata()
            .await
            .expect("unable to read file metadata");
        debug_assert_eq!(0, metadata.len(), "file size should still be 0 bytes")
    }

    #[test]
    fn test_open_cache_sync() {
        let tmp_dir = TempDir::new().expect("failed to create temp directory");
        let dir_path = tmp_dir.path();
        debug_assert!(
            !dir_path.join("metadata.mpk").exists(),
            "metadata file should not exist yet"
        );
        let fixture = FileSystemCacher::open_sync(dir_path).expect("failed to open cache");
        debug_assert_eq!(dir_path, &fixture.cache_dir, "cache dir is expected");
        debug_assert!(
            dir_path.join("metadata.mpk").exists(),
            "metadata file should exist now"
        );
        let metadata = std::fs::metadata(fixture.cache_dir.join(METADATA_FILE_NAME))
            .expect("unable to read file metadata");
        debug_assert_eq!(0, metadata.len(), "file size should still be 0 bytes")
    }

    #[tokio::test]
    async fn test_save_metadata() {
        let tmp_dir = TempDir::new().expect("failed to create temp directory");
        let dir_path = tmp_dir.path();
        debug_assert!(
            !dir_path.join("metadata.mpk").exists(),
            "metadata file should not exist yet"
        );
        let fixture = FileSystemCacher::open(dir_path)
            .await
            .expect("failed to open cache");
        debug_assert_eq!(dir_path, &fixture.cache_dir, "cache dir is expected");
        debug_assert!(
            dir_path.join("metadata.mpk").exists(),
            "metadata file should exist now"
        );

        fixture
            .save_metadata(
                String::from("asdfasdfasdf"),
                crate::Entry {
                    ts: Utc::now(),
                    ttl: 60,
                },
            )
            .await
            .expect("failed to save metadata");

        let metadata = fixture
            .metadata
            .lock()
            .await
            .0
            .metadata()
            .await
            .expect("unable to read file metadata");
        debug_assert_eq!(25, metadata.len(), "file size should be 25 bytes");

        fixture
            .save_metadata(
                String::from("foo"),
                crate::Entry {
                    ts: Utc::now(),
                    ttl: 60,
                },
            )
            .await
            .expect("failed to save metadata");

        let metadata = fixture
            .metadata
            .lock()
            .await
            .0
            .metadata()
            .await
            .expect("unable to read file metadata");
        debug_assert_eq!(40, metadata.len(), "file size should be n bytes");
    }

    #[tokio::test]
    async fn test_load_existing_cache() {
        let tmp_dir = TempDir::new().expect("failed to create temp directory");
        let dir_path = tmp_dir.path();

        // Setup Test Data
        tokio::fs::write(
            dir_path.join(METADATA_FILE_NAME),
            base64::prelude::BASE64_STANDARD
                .decode("gaRhc2Rmks8AAAGZOeqCeM0OEA==")
                .expect("unable to decode"),
        )
        .await
        .expect("failed to setup metadata file");

        debug_assert!(
            dir_path.join(METADATA_FILE_NAME).exists(),
            "metadata file should exist"
        );
        let fixture = FileSystemCacher::open(dir_path)
            .await
            .expect("failed to open cache");
        let entry = fixture
            .get_metadata("asdf".to_string())
            .await
            .expect("ffff");
        debug_assert!(entry.is_some(), "did not find entry");
        let entry = entry.unwrap();
        debug_assert_eq!(entry.ttl, 3600, "not correct ttl");
    }

    #[tokio::test]
    async fn test_cache_file() {
        let tmp_dir = TempDir::new().expect("failed to create temp directory");
        let dir_path = tmp_dir.path();

        let fixture = FileSystemCacher::open(dir_path)
            .await
            .expect("failed to open cache");

        {
            let meta = &fixture.metadata.lock().await.1;
            debug_assert!(meta.is_empty(), "meta should be empty");
        }

        let f = {
            let mut entries = tokio::fs::read_dir(dir_path)
                .await
                .expect("unable to create dir entry iterator");
            let mut items = vec![];
            while let Some(entry) = entries
                .next_entry()
                .await
                .expect("unable to look at next dir entry")
            {
                items.push(entry.file_name().to_str().unwrap().to_string());
            }
            items.sort(); // read_dir order is unspecified
            items
        };
        debug_assert_eq!(vec![METADATA_FILE_NAME], f, "");

        fixture
            .save_cache("http://localhost:1234/my/resource?foo=bar", b"{\"data\":3}")
            .await
            .expect("failed to cache value");

        let f = {
            let mut entries = tokio::fs::read_dir(dir_path)
                .await
                .expect("unable to create dir entry iterator");
            let mut items = vec![];
            while let Some(entry) = entries
                .next_entry()
                .await
                .expect("unable to look at next dir entry")
            {
                items.push(entry.file_name().to_str().unwrap().to_string());
            }
            items.sort(); // read_dir order is unspecified
            items
        };
        let mut expected = vec![
            METADATA_FILE_NAME,
            "7cd876f5534890e60e293c92f2ed208cde2fdaa77a99dcdbb5a88f06992ad6d3",
        ];
        expected.sort();
        debug_assert_eq!(expected, f, "should have the new file");
        {
            let meta = &fixture.metadata.lock().await.1;
            debug_assert!(!meta.is_empty(), "meta should not be empty");
        }
    }
}
