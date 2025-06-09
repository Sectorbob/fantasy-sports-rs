use std::path::PathBuf;

use crate::cacheable::Cacheable;

pub struct FileSystemCacher {
    cache_dir: PathBuf
}
impl FileSystemCacher {
    pub fn new<T: Into<PathBuf>>(cache_dir: T) -> Self {
        FileSystemCacher { cache_dir: cache_dir.into() }
    }
}
impl Cacheable for FileSystemCacher {
    fn cache_dir(&self) -> std::path::PathBuf {
        self.cache_dir.clone()
    }
}
