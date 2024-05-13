use std::path::{Path, PathBuf};

use chrono::{DateTime, Local, Utc};
use configparser::ini::{Ini, IniDefault, WriteOptions};
use derive_builder::Builder;
use log::info;
use serde::{Deserialize, Deserializer, Serialize};

use crate::{EnvVarsStyle, DEFAULT_REGION};

pub type Result<T> = std::result::Result<T, ConfigError>;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error(transparent)]
    ConfigFilePathError(#[from] std::io::Error),

    #[error("Failed to parse config file")]
    ConfigLoadError(String),

    #[error("Failed to expand variable '{0}' in the path '{1}'")]
    PathExpansionError(String, String),
}

#[derive(Clone, Debug)]
pub enum OutputFormat {
    Json,
    CredentialsFile {
        config_file: String,
        credentials_file: String,
        profile: String,
    },
    EnvVars(EnvVarsStyle),
}

#[derive(Debug, Clone, Copy)]
struct WriteOptionsBuilder {
    space_around_delimiters: bool,
    multiline_line_indentation: usize,
    blank_lines_between_sections: usize,
}

impl WriteOptionsBuilder {
    fn new() -> Self {
        let default_write_options = &WriteOptions::default();
        Self {
            space_around_delimiters: default_write_options.space_around_delimiters,
            multiline_line_indentation: default_write_options.multiline_line_indentation,
            blank_lines_between_sections: default_write_options.blank_lines_between_sections,
        }
    }

    fn space_around_delimiters(&mut self, space_around_delimiters: bool) -> &mut Self {
        self.space_around_delimiters = space_around_delimiters;
        self
    }

    fn multiline_line_indentation(&mut self, multiline_line_indentation: usize) -> &mut Self {
        self.multiline_line_indentation = multiline_line_indentation;
        self
    }

    fn blank_lines_between_sections(&mut self, blank_lines_between_sections: usize) -> &mut Self {
        self.blank_lines_between_sections = blank_lines_between_sections;
        self
    }

    fn build(self) -> WriteOptions {
        WriteOptions::new_with_params(
            self.space_around_delimiters,
            self.multiline_line_indentation,
            self.blank_lines_between_sections,
        )
    }
}

fn checked_deserialize_expiration<'de, D>(
    deserializer: D,
) -> std::result::Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let expiration = DateTime::<Utc>::deserialize(deserializer)?;
    if expiration < Utc::now() {
        return Err(serde::de::Error::custom(format!(
            "AWS credentials expired at {expiration} ({} local time)",
            expiration.with_timezone(&Local)
        )));
    }
    Ok(expiration)
}

#[derive(Clone, Serialize, Deserialize, Debug, Builder)]
#[builder(build_fn(validate = "Self::validate"))]
#[serde(rename_all = "PascalCase")]
pub struct TemporaryAwsCredentials {
    pub version: i32,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: String,
    #[serde(deserialize_with = "checked_deserialize_expiration")]
    #[builder(try_setter, setter(into))]
    /// If expiration is set by the builder or by `serde` deserialization and it is in the past then an error will be returned
    pub expiration: DateTime<Utc>,
    #[serde(skip)]
    #[serde(default = "default_region")]
    pub region: String,
}

impl TemporaryAwsCredentialsBuilder {
    fn validate(&self) -> std::result::Result<(), String> {
        match self.expiration {
            Some(expiration) if expiration < Utc::now() => Err(format!(
                "AWS credentials expired at {expiration} ({} local time)",
                expiration.with_timezone(&Local)
            )),
            _ => Ok(()),
        }
    }
}

impl TemporaryAwsCredentials {
    pub fn output_as(&self, output_format: &OutputFormat) -> std::io::Result<()> {
        match output_format {
            OutputFormat::Json => {
                self.as_json();
                Ok(())
            }
            OutputFormat::CredentialsFile {
                config_file,
                credentials_file,
                profile,
            } => self
                .to_credentials_file(config_file, credentials_file, profile)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)),
            OutputFormat::EnvVars(style) => {
                self.as_env_vars(style);
                Ok(())
            }
        }
    }
    fn as_json(&self) {
        let json_string = serde_json::to_string_pretty(&self).unwrap(); // TODO: handle error
        println!("{json_string}");
        info!(
            "Credentials expire at {}",
            self.expiration.with_timezone(&Local)
        );
    }

    fn to_credentials_file(
        &self,
        config_file: impl AsRef<str>,
        credentials_file: impl AsRef<str>,
        profile: impl AsRef<str>,
    ) -> Result<()> {
        let aws_config_file_path = path_for_file(config_file.as_ref())?;
        let mut aws_config = ini_for_file(&aws_config_file_path)?;

        let aws_credentials_file_path = path_for_file(credentials_file.as_ref())?;
        let mut aws_credentials = ini_for_file(&aws_credentials_file_path)?;

        aws_config.set(profile.as_ref(), "region", Some(self.region.clone()));

        aws_credentials.set(
            profile.as_ref(),
            "aws_access_key_id",
            Some(self.access_key_id.clone()),
        );
        aws_credentials.set(
            profile.as_ref(),
            "aws_secret_access_key",
            Some(self.secret_access_key.clone()),
        );
        aws_credentials.set(
            profile.as_ref(),
            "aws_session_token",
            Some(self.session_token.clone()),
        );

        let write_options = WriteOptionsBuilder::new()
            .space_around_delimiters(true)
            .multiline_line_indentation(2)
            .blank_lines_between_sections(1)
            .build();

        aws_config.pretty_write(aws_config_file_path, &write_options)?;
        aws_credentials.pretty_write(aws_credentials_file_path, &write_options)?;

        Ok(())
    }

    fn as_env_vars(&self, style: &crate::EnvVarsStyle) {
        match style {
            EnvVarsStyle::Sh => {
                println!(
                    r#"export AWS_ACCESS_KEY_ID={}
export AWS_SECRET_ACCESS_KEY={}
export AWS_SESSION_TOKEN={}
export AWS_REGION={}
export AWS_DEFAULT_REGION={DEFAULT_REGION}
"#,
                    self.access_key_id, self.secret_access_key, self.session_token, self.region
                )
            }
            EnvVarsStyle::PowerShell => {
                println!(
                    r#"$env:AWS_ACCESS_KEY_ID="{}"
$env:AWS_SECRET_ACCESS_KEY="{}"
$env:AWS_SESSION_TOKEN="{}"
$env:AWS_REGION="{}"
$env:AWS_DEFAULT_REGION="{DEFAULT_REGION}""#,
                    self.access_key_id, self.secret_access_key, self.session_token, self.region
                )
            }
        }
    }
}

fn default_region() -> String {
    DEFAULT_REGION.to_string()
}

fn create_if_not_exists(file_path: &Path) -> Result<()> {
    if !file_path.exists() {
        // We assume that the last part of the path is a file
        // Create the file's parent directory if needed
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::File::create(file_path)?; // TODO: Set correct permissions on file. They should be 600 on Unix-y systems
    }

    Ok(())
}

fn path_for_file(file: &str) -> Result<PathBuf> {
    let file_path = shellexpand::full(&file)
        .map_err(|e| ConfigError::PathExpansionError(e.var_name, file.to_string()))?
        .to_string();
    Ok(Path::new(&file_path).to_path_buf())
}

fn ini_for_file(file: &Path) -> Result<Ini> {
    create_if_not_exists(file)?;

    let mut ini_defaults = IniDefault::default();
    ini_defaults.multiline = true;
    ini_defaults.default_section = String::from("no-section");
    ini_defaults.case_sensitive = true;

    let mut aws_credentials = Ini::new_from_defaults(ini_defaults.clone());
    aws_credentials
        .load(file)
        .map_err(ConfigError::ConfigLoadError)?;
    Ok(aws_credentials)
}
