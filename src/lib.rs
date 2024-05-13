mod assume;
mod cache;
mod cli;
mod defaults;
mod models;

use std::error::Error;

use inquire::Confirm;
use log::error;
use log::{info, warn};

use cache::{CachedCredentialsError, CredentialsCache};
use defaults::{DEFAULT_CREDS_VERSION, DEFAULT_REGION};
use models::{RoleInfo, RoleInfoBuilder};

use clap::{Command, CommandFactory, Parser};

use cli::{CacheCommands, Cli, Commands, OutputAsCommands};

use crate::assume::models::OutputFormat;

fn print_completions<G, W>(gen: G, cmd: &mut Command, output: &mut W)
where
    G: clap_complete::Generator,
    W: std::io::Write,
{
    clap_complete::generate(gen, cmd, cmd.get_name().to_string(), output);
}

#[derive(Clone, Debug)]
enum EnvVarsStyle {
    Sh,
    PowerShell,
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
                    match CredentialsCache::directory() {
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
                        CredentialsCache::remove_all_cached_files()?;
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
                OutputAsCommands::EnvVars { style } => {
                    OutputFormat::EnvVars(match style.as_str() {
                        "sh" => EnvVarsStyle::Sh,
                        "powershell" => EnvVarsStyle::PowerShell,
                        _ => {
                            error!("Unsupported shell type: {style}");
                            return Ok(());
                        }
                    })
                }
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

            let credentials_cache = CredentialsCache::new(&role_info)?;
            let cached_credentials = if force {
                None
            } else {
                info!("Attempting to fetch credentials from cache");
                match credentials_cache.credentials() {
                    Ok(credentials) => Some(credentials),
                    Err(error) => {
                        match error {
                            CachedCredentialsError::JsonError(err) => {
                                warn!("JSON data error: {err}. Ignoring cache.")
                            }
                            CachedCredentialsError::FileSystemError(_) => {
                                info!("Cache file not found")
                            }
                            CachedCredentialsError::UnsupportedPlatform => {
                                warn!("Can not cache credentials on this platform")
                            }
                        };
                        None
                    }
                }
            };

            let credentials = match cached_credentials {
                Some(credentials) => credentials,
                None => {
                    info!("Acquiring credentials");
                    let new_credentials = assume::acquire_credentials(&role_info).await?;
                    credentials_cache.store_credentials(&new_credentials)?;
                    new_credentials
                }
            };
            credentials.output_as(&output_format)?;
        }
        Commands::GenerateCompletions { shell, mut output } => {
            eprintln!("Generating completion file for {shell} ...");
            let mut cmd = Cli::command();
            print_completions(shell, &mut cmd, &mut output);
        }
    }
    Ok(())
}
