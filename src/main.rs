use std::error::Error;
use std::fs::File;
use log::{debug, info, warn, error};
use std::path::Path;
use std::path::PathBuf;
use chrono::{Utc, Local};
use derive_builder::Builder;
use cache::{CachedCredentialsError, create_cache_dir, cache_dir, cache_file_path, remove_all_cached_files};
use defaults::{DEFAULT_REGION, DEFAULT_CREDS_VERSION};
use assume::models::TemporaryAwsCredentials;

mod assume;
mod cache;
mod cli;
mod defaults;

#[derive(Debug, Builder)]
pub struct RoleInfo {
    role_name: String,
    account_id: String,
    region: String,
    duration: i32,
}

impl RoleInfo {
    pub fn role_arn(&self) -> String {
        format!("arn:aws:iam::{}:role/{}", self.account_id, self.role_name)
    }
}

fn store_credentials_cache(cache_file_path: &Path, credentials: &TemporaryAwsCredentials) -> Result<(), CachedCredentialsError> {
    debug!("Storing creds to file {}", cache_file_path.as_os_str().to_str().unwrap());
    let cache_file = File::create(cache_file_path)?;
    serde_json::to_writer_pretty(cache_file, &credentials)?;
    Ok(())
}

fn credentials_from_cache(path: PathBuf) -> Result<TemporaryAwsCredentials, CachedCredentialsError> {
    let file = File::open(path)?;
    let credentials: TemporaryAwsCredentials = serde_json::from_reader(file)?;
    if credentials.expiration < Utc::now() {
        return Err(CachedCredentialsError::TokenExpired(credentials.expiration))
    }
    Ok(credentials)
}
fn print_credentials(credentials: &TemporaryAwsCredentials, output_format: OutputFormat) {
    match output_format {
        OutputFormat::Json => credentials.as_json(),
        OutputFormat::EnvVars => credentials.as_env_vars(),
    }
}

#[derive(Clone, Debug)]
enum OutputFormat {
    Json,
    EnvVars,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = cli::cli().get_matches();

    let verbosity = cli::verbosity(&matches);
    env_logger::Builder::new()
        .filter_level(verbosity.log_level_filter())
        .init();

    match matches.subcommand() {
        Some(("cache", sub_matches)) => {
            let cache_command = sub_matches.subcommand().unwrap_or(("path", sub_matches));
            match cache_command {
                ("path", _) => {
                    println!("{}", cache_dir().to_str().unwrap());
                    return Ok(());
                } 
                ("clear", sub_matches) => {
                    let delete_unconditionally = *sub_matches.get_one::<bool>("yes").unwrap();
                    let mut do_delete = false;
                    if delete_unconditionally {
                        do_delete = true;
                    } else {
                        print!("This will delete all cached credentials in the cache directory. Type yes to continue, anything else to cancel: ");
                        let answer: String = text_io::read!();
                        if answer.to_lowercase() == "yes" {
                            do_delete = true;
                        }
                    }
                    if do_delete {
                        remove_all_cached_files()?;
                    }

                }
                (name, _) => {
                    unreachable!("Unsupported subcommand '{name}'")
                }
            }
        }
        Some(("assume", sub_matches)) => {
            create_cache_dir()?;

            let account_id = sub_matches.get_one::<String>("account").unwrap().to_string();
            let role_name = sub_matches.get_one::<String>("role").unwrap().to_string();
            let region = sub_matches.get_one::<String>("region").unwrap().to_string();
            let duration = *sub_matches.get_one::<i32>("duration").unwrap();
            let mut output_format = OutputFormat::Json;
            if let Some(env_vars_requested) = sub_matches.get_one::<bool>("env_vars") {
                output_format = if *env_vars_requested {
                    OutputFormat::EnvVars
                } else {
                    OutputFormat::Json
                }
            }
            let force_renew = *sub_matches.get_one::<bool>("force").unwrap();
            info!("Using duration {duration}");
            info!("Using region {region}");

            let role_info = RoleInfoBuilder::default()
                .role_name(role_name)
                .account_id(account_id.clone())
                .region(region)
                .duration(duration)
                .build()?;

            if !force_renew {
                info!("Attempting to fetch credentials from cache");
                match credentials_from_cache(cache_file_path(&role_info)) {
                    Ok(credentials) => {
                        print_credentials(&credentials, output_format);
                        return Ok(());
                    }
                    Err(error) => match error {
                        CachedCredentialsError::JsonError(err) => warn!("JSON error: {err}"),
                        CachedCredentialsError::FileError(err) => warn!("File error: {err}"),
                        CachedCredentialsError::TokenExpired(expiration_time) => warn!("AWS credentials expired at {expiration_time} ({} local time)", expiration_time.with_timezone(&Local)),
                    }
                };
            }

            info!("Acquiring credentials");
            let credentials = assume::acquire_credentials(&role_info).await?;
            print_credentials(&credentials, output_format);
            store_credentials_cache(&cache_file_path(&role_info), &credentials)?;
        }
        _ => {
            error!("Unknown or no subcommand");
        }
    }

    Ok(())
}
