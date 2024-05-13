use std::fs::File;

use log::debug;

use super::assume::models::TemporaryAwsCredentials;
use super::models::RoleInfo;

pub type Result<T> = std::result::Result<T, CachedCredentialsError>;

#[derive(Debug, thiserror::Error)]
pub enum CachedCredentialsError {
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    FileSystemError(#[from] std::io::Error),
    #[error("Unsupported platform")]
    UnsupportedPlatform,
}

#[derive(Debug)]
pub struct CredentialsCache {
    cache_file_path: std::path::PathBuf,
}

impl CredentialsCache {
    pub fn new(role_info: &RoleInfo) -> Result<Self> {
        let filename = format!(
            "{account_id}-{role_name}.creds",
            account_id = role_info.account_id,
            role_name = role_info.role_name
        );
        Self::create_cache_dir()?;

        let cache_file_path = Self::directory()?.join(filename);
        let cache = Self {
            cache_file_path,
        };
        Ok(cache)
    }

    pub fn directory() -> Result<std::path::PathBuf> {
        let pkg_name = env!("CARGO_PKG_NAME");
        let dir = dirs::cache_dir()
            .ok_or(CachedCredentialsError::UnsupportedPlatform)?
            .join(pkg_name);
        Ok(dir)
    }

    pub fn store_credentials(&self, credentials: &TemporaryAwsCredentials) -> Result<()> {
        debug!(
            "Storing creds to file {}",
            self.cache_file_path.as_os_str().to_str().unwrap()
        );
        let cache_file = File::create(&self.cache_file_path)?;
        serde_json::to_writer_pretty(cache_file, &credentials)?;
        Ok(())
    }

    pub fn credentials(&self) -> Result<TemporaryAwsCredentials> {
        let file = std::fs::File::open(&self.cache_file_path)?;
        let credentials: TemporaryAwsCredentials = serde_json::from_reader(file)?;
        Ok(credentials)
    }

    pub fn remove_all_cached_files() -> Result<()> {
        for entry in std::fs::read_dir(Self::directory()?)? {
            log::info!("Deleting cache file {:?}", entry);
            std::fs::remove_file(entry?.path())?;
        }
        Ok(())
    }

    fn create_cache_dir() -> Result<()> {
        std::fs::create_dir_all(Self::directory()?)?;
        Ok(())
    }
}
