use clap::{Args, ArgMatches, ArgAction, value_parser, arg, Command, FromArgMatches};
use clap_verbosity_flag::Verbosity;

use super::defaults::{DEFAULT_DURATION, DEFAULT_REGION};

#[derive(Args, Debug)]
pub struct VerbosityArgs {
    #[command(flatten)]
    verbose: Verbosity,
}

pub fn cli() -> Command {
    let cli = Command::new("aws-credentials-cli")
        .about("Utility to acquire temporary AWS credentials using the Azure AD based token exchange method.")
        .subcommand_required(true)
        .arg_required_else_help(true)
        // Add option for setting custom cache dir and retain the option between calls
        // Add option for storing and getting credentials from the AWS creds file. Use the aws_cred
        // crate for that.
        .subcommand(
    Command::new("cache")
                .about("Credentials cache operations. If no subcommand is given then it defaults to 'path'.")
                .subcommand(
                    Command::new("path")
                    .about("Prints the cache directory path")
                )
                .subcommand(
                    Command::new("clear")
                    .about("Clear the credentials cache. Deletes all files in the cache directory.")
                    .arg(arg!(-y --yes "Do not ask for permission. Use with caution!")
                         .action(ArgAction::SetTrue)
                         .default_value("false"))
                )
        )
        .subcommand(
    Command::new("assume")
                .about("Assume role on account to get temporary credentials.")
                .arg_required_else_help(true)
                .arg(arg!(-a --account <ACCOUNT_ID>)
                     .required(true)
                )
                .arg(arg!(-r --role <ROLE_NAME>)
                     .required(true)
                )
                .arg(arg!(-d --duration <SECONDS> "The AWS session duration in seconds. Must be minimum 900 seconds (15 minutes).")
                    .default_value(DEFAULT_DURATION)
                    .value_parser(value_parser!(i32))
                )
                .arg(arg!(-g --region <REGION> "The region to use.")
                     .default_value(DEFAULT_REGION)
                 )
                .arg(arg!(-f --force "Force fetching new credentials regardless of non-expired cached credentials.")
                     .action(ArgAction::SetTrue)
                     .default_value("false")
                )
                .arg(arg!(-j --json "Output as JSON.")
                     .action(ArgAction::SetTrue)
                     .default_value("true")
                     .conflicts_with("env_vars")
                )
                .arg(arg!(-e --env_vars "Output as shell variable export statements suitable for shell eval.")
                     .action(ArgAction::SetTrue)
                     .default_value("false")
                )
        );
    VerbosityArgs::augment_args(cli)
}

pub fn verbosity(matches: &ArgMatches) -> Verbosity {
    VerbosityArgs::from_arg_matches(matches).unwrap().verbose
}
