mod assume;
mod cache;
mod cli;
mod defaults;
mod models;

use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use chrono::{Utc, Local};
use log::{debug, info, warn};

use cache::{
    CachedCredentialsError,
    cache_file_path,
    create_cache_dir,
    cache_dir,
    remove_all_cached_files,
};
use defaults::{DEFAULT_REGION, DEFAULT_CREDS_VERSION};
use assume::models::TemporaryAwsCredentials;
use models::{RoleInfo, RoleInfoBuilder};

use clap::{Command, CommandFactory, Parser};
use clap_complete::{generate, Generator};
use cli::{Cli, Commands, CacheCommands};

fn print_completions<G, W>(gen: G, cmd: &mut Command, output: &mut W) 
where
    G: Generator,
    W: Write,
{
    generate(gen, cmd, cmd.get_name().to_string(), output);
}

fn store_credentials_cache(cache_file_path: &Path, credentials: &TemporaryAwsCredentials) -> cache::Result<()> {
    debug!("Storing creds to file {}", cache_file_path.as_os_str().to_str().unwrap());
    create_cache_dir()?;
    let cache_file = File::create(cache_file_path)?;
    serde_json::to_writer_pretty(cache_file, &credentials)?;
    Ok(())
}

fn credentials_from_cache(path: PathBuf) -> cache::Result<TemporaryAwsCredentials> {
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

pub async fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();
    match cli.command {
        Commands::Cache { command } => {
            let cache_command = command.unwrap_or(CacheCommands::Path);

            match cache_command {
                CacheCommands::Path => {
                    match cache_dir() {
                        Ok(cache_dir) => {
                            println!("{}", cache_dir.display());
                        }
                        Err(error) => {
                            eprintln!("{error}");
                        }
                    }
                    return Ok(());
                }
                CacheCommands::Clear { yes } => {
                    let do_delete = if yes {
                        true
                    } else {
                        print!("This will delete all cached credentials in the cache directory. Type yes to continue, anything else to cancel: ");
                        let answer: String = text_io::read!();
                        answer.to_lowercase() == "yes"
                    };
                    if do_delete {
                        remove_all_cached_files()?;
                    }
                }
            }
        }
        Commands::Assume {
            aws_partition,
            account,
            role,
            duration,
            region,
            force,
            json: _,
            env_vars
        } => {
            let output_format = if env_vars {
                OutputFormat::EnvVars
            } else {
                OutputFormat::Json
            };
            info!("Using duration {duration}");
            info!("Using region {region}");

            let role_info = RoleInfoBuilder::default()
                .aws_partition(aws_partition)
                .role_name(role)
                .account_id(account)
                .region(region)
                .duration(duration)
                .build()?;
            if !force {
                info!("Attempting to fetch credentials from cache");
                let file_path = cache_file_path(&role_info)?;
                match credentials_from_cache(file_path) {
                    Ok(credentials) => {
                        print_credentials(&credentials, output_format);
                        return Ok(());
                    }
                    Err(error) => match error {
                        CachedCredentialsError::JsonError(err) => warn!("JSON error: {err}"),
                        CachedCredentialsError::FileSystemError(err) => warn!("File error: {err}"),
                        CachedCredentialsError::TokenExpired(expiration_time) => warn!("AWS credentials expired at {expiration_time} ({} local time)", expiration_time.with_timezone(&Local)),
                        CachedCredentialsError::UnsupportedPlatform => warn!("Can not cache credentials on this platform")
                    }
                };
            }

            info!("Acquiring credentials");
            let credentials = assume::acquire_credentials(&role_info).await?;
            print_credentials(&credentials, output_format);
            store_credentials_cache(&cache_file_path(&role_info)?, &credentials)?;
        }
        Commands::GenerateCompletions { shell, mut output } => {
            eprintln!("Generating completion file for {shell} ...");
            let mut cmd = Cli::command();
            print_completions(shell, &mut cmd, &mut output);
        }
    }
    Ok(())
}
