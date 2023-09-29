use std::fs;
use std::io;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use log::{info, error};

use super::RoleInfo;

#[derive(Debug, thiserror::Error)]
pub enum CachedCredentialsError {
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    FileError(#[from] std::io::Error),
    #[error("Cached AWS credentials expired at {0}")]
    TokenExpired(DateTime<Utc>),
}

pub fn cache_dir() -> PathBuf {
    let pkg_name = env!("CARGO_PKG_NAME");
    dirs::cache_dir().unwrap().join(pkg_name)
}

pub fn create_cache_dir() -> io::Result<()> {
    fs::create_dir_all(cache_dir())?;
    Ok(())
}

pub fn cache_file_path(role_info: &RoleInfo) -> PathBuf {
    let filename = format!("{account_id}-{role_name}.creds", account_id = role_info.account_id, role_name = role_info.role_name);
    cache_dir().join(filename)
}

pub fn remove_all_cached_files() -> io::Result<()> {
    let path = cache_dir();
    for entry in fs::read_dir(path)? {
        info!("Deleting cache file {:?}", entry);
        fs::remove_file(entry?.path())?;
    }
    Ok(())
}
