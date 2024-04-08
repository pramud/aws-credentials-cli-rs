mod assume;
mod cache;
mod cli;
mod defaults;
mod models;

use std::error::Error;
use std::fs::File;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use chrono::{Local, Utc};
use inquire::Confirm;
use log::error;
use log::{debug, info, warn};

use assume::models::TemporaryAwsCredentials;
use cache::{
    cache_dir, cache_file_path, create_cache_dir, remove_all_cached_files, CachedCredentialsError,
};
use defaults::{DEFAULT_CREDS_VERSION, DEFAULT_REGION};
use models::{RoleInfo, RoleInfoBuilder};

use clap::{Command, CommandFactory, Parser};

use cli::{CacheCommands, Cli, Commands, OutputAsCommands};

fn print_completions<G, W>(gen: G, cmd: &mut Command, output: &mut W)
where
    G: clap_complete::Generator,
    W: std::io::Write,
{
    clap_complete::generate(gen, cmd, cmd.get_name().to_string(), output);
}

fn store_credentials_cache(
    cache_file_path: &Path,
    credentials: &TemporaryAwsCredentials,
) -> cache::Result<()> {
    debug!(
        "Storing creds to file {}",
        cache_file_path.as_os_str().to_str().unwrap()
    );
    create_cache_dir()?;
    let cache_file = File::create(cache_file_path)?;
    serde_json::to_writer_pretty(cache_file, &credentials)?;
    Ok(())
}

fn credentials_from_cache(path: PathBuf) -> cache::Result<TemporaryAwsCredentials> {
    let file = File::open(path)?;
    let credentials: TemporaryAwsCredentials = serde_json::from_reader(file)?;
    if credentials.expiration < Utc::now() {
        return Err(CachedCredentialsError::TokenExpired(credentials.expiration));
    }
    Ok(credentials)
}

fn print_credentials(credentials: &TemporaryAwsCredentials, output_format: OutputFormat) -> io::Result<()> {
    match output_format {
        OutputFormat::Json => Ok(credentials.as_json()),
        OutputFormat::CredentialsFile {
            config_file,
            credentials_file,
            profile,
        } => credentials.to_credentials_file(&config_file, &credentials_file, &profile).map_err(|e| io::Error::new(io::ErrorKind::Other, e)),
        OutputFormat::EnvVars => Ok(credentials.as_env_vars()),
    }
}

#[derive(Clone, Debug)]
enum OutputFormat {
    Json,
    CredentialsFile {
        config_file: String,
        credentials_file: String,
        profile: String,
    },
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
                        Confirm::new("About to delete cached credentials. Are you sure?")
                            .with_default(false)
                            .with_help_message(
                                "This will delete ALL cached credentials in the cache directory.",
                            )
                            .with_placeholder("y/yes or n/no")
                            .prompt()
                            .unwrap_or(false)
                    };
                    if do_delete {
                        info!("Deleting all cached credentials.");
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
            output_as,
        } => {
            let output_command = output_as.unwrap_or(OutputAsCommands::Json);

            let output_format = match output_command {
                OutputAsCommands::Json => OutputFormat::Json,
                OutputAsCommands::CredentialsFile {
                    config_file,
                    credentials_file,
                    profile,
                } => OutputFormat::CredentialsFile {
                    config_file,
                    credentials_file,
                    profile,
                },
                OutputAsCommands::EnvVars => OutputFormat::EnvVars,
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
                        print_credentials(&credentials, output_format.clone())?;
                        return Ok(());
                    }
                    Err(error) => match error {
                        CachedCredentialsError::JsonError(err) => error!("JSON error: {err}"),
                        CachedCredentialsError::FileSystemError(err) => error!("File error: {err}"),
                        CachedCredentialsError::TokenExpired(expiration_time) => warn!(
                            "AWS credentials expired at {expiration_time} ({} local time)",
                            expiration_time.with_timezone(&Local)
                        ),
                        CachedCredentialsError::UnsupportedPlatform => {
                            warn!("Can not cache credentials on this platform")
                        }
                    },
                };
            }

            info!("Acquiring credentials");
            let credentials = assume::acquire_credentials(&role_info).await?;
            print_credentials(&credentials, output_format)?;
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
