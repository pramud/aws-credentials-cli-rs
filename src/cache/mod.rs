use chrono::{DateTime, Utc};

use super::models::RoleInfo;

pub type Result<T> = std::result::Result<T, CachedCredentialsError>;

#[derive(Debug, thiserror::Error)]
pub enum CachedCredentialsError {
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    FileSystemError(#[from] std::io::Error),
    #[error("Cached AWS credentials expired at {0}")]
    TokenExpired(DateTime<Utc>),
}

pub fn cache_dir() -> std::path::PathBuf {
    let pkg_name = env!("CARGO_PKG_NAME");
    dirs::cache_dir().map(|h| h.join(pkg_name)).unwrap()
}

pub fn create_cache_dir() -> Result<()> {
    std::fs::create_dir_all(cache_dir())?;
    Ok(())
}

pub fn cache_file_path(role_info: &RoleInfo) -> std::path::PathBuf {
    let filename = format!("{account_id}-{role_name}.creds", account_id = role_info.account_id, role_name = role_info.role_name);
    cache_dir().join(filename)
}

pub fn remove_all_cached_files() -> Result<()> {
    let path = cache_dir();
    for entry in std::fs::read_dir(path)? {
        log::info!("Deleting cache file {:?}", entry);
        std::fs::remove_file(entry?.path())?;
    }
    Ok(())
}
