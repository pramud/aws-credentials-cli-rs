use clap::{Parser, Subcommand, ValueHint};
use clap_complete::Shell;
use clap_verbosity_flag::Verbosity;

use crate::defaults::{
    DEFAULT_DURATION,
    VALID_AWS_PARTITIONS,
    DEFAULT_AWS_PARTITION,
    DEFAULT_REGION,
    ENV_VARS_STYLES,
};

#[derive(Debug, Parser)]
#[command(name="aws-credentials-cli")]
#[command(about="Utility to acquire temporary AWS credentials using the Azure AD based token exchange method.", long_about = None)]
pub struct Cli {
    // TODO:
    // Add subcommand for config
    //     Add config subcommand for setting custom cache dir
    // Add option for storing and getting credentials from the AWS creds file.

    #[command(flatten)]
    pub verbose: Verbosity,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Credentials cache operations. If no subcommand is given then it defaults to 'path'.
    Cache {
        #[command(subcommand)]
        command: Option<CacheCommands>,
    },
    /// Assume role on account to get temporary credentials.
    Assume {
        /// The AWS partition for the account
        #[arg(long)]
        #[arg(value_parser = VALID_AWS_PARTITIONS)]
        #[arg(default_value_t = String::from(DEFAULT_AWS_PARTITION))]
        aws_partition: String,

        /// Assume role on account to get temporary credentials.
        #[arg(short, long)]
        account: String,

        /// The role to assume.
        #[arg(short, long)]
        role: String,

        /// The AWS session duration in seconds. Must be minimum 900 seconds (15 minutes).
        #[arg(short, long)]
        #[arg(value_parser = clap::value_parser!(i32).range(900..))]
        #[arg(default_value_t = DEFAULT_DURATION)]
        duration: i32,

        /// The region to use.
        #[arg(long, default_value_t = DEFAULT_REGION.to_string())]
        region: String,

        /// Force fetching new credentials regardless of non-expired cached credentials.
        #[arg(short, long)]
        force: bool,

        /// Output format
        #[command(subcommand)]
        output_as: Option<OutputAsCommands>,
        // #[arg(short, long, default_value = "json")]
        // #[arg(value_parser = ["json", "credentials-file", "env-vars"])]
        // output: String,
    },
    /// Generate completion scripts for a supported shell.
    /// Redirect the output to a suitable directory for your shell and run the intitialization
    /// command for the completion system in the selected shell.
    GenerateCompletions {
        /// Write output to this file. Use `-` for standard output (default).
        #[arg(short, long, value_hint = ValueHint::FilePath)]
        #[arg(value_parser, default_value="-")]
        output: clio::Output,

        /// The shell for which to generate completion scripts.
        #[arg(value_parser = clap::value_parser!(Shell))]
        shell: Shell,
    }
}

#[derive(Debug, Subcommand)]
pub enum CacheCommands {
    /// Clears the credentials cache. Deletes all files in the cache directory.
    Clear {
        /// Do not ask for permission.
        #[arg(short, long)]
        yes: bool,
    },
    /// Prints the cache directory path.
    Path,
}

#[derive(Debug, Subcommand)]
pub enum OutputAsCommands {
    /// Output to standard output.
    Json,
    /// Output to the AWS credentials file.
    CredentialsFile {
        /// The config file to write to.
        /// If the file does not exist it will be created.
        #[arg(long, default_value = "~/.aws/config")]
        config_file: String,

        /// The AWS credentials file to write to.
        #[arg(long, default_value = "~/.aws/credentials")]
        credentials_file: String,

        /// The profile to write to.
        #[arg(short, long, default_value = "default")]
        profile: String,
    },
    /// Output to environment variables.
    EnvVars {
        /// The shell type to output for.
        #[arg(short, long, value_parser = ENV_VARS_STYLES, default_value = "sh")]
        style: String,
    },
}
