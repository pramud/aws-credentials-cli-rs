use chrono::{DateTime, Local, Utc};
use configparser::ini::{Ini, WriteOptions};
use derive_builder::Builder;
use expanduser::expanduser;
use log::info;
use serde::{Deserialize, Serialize};

use crate::DEFAULT_REGION;

pub type Result<T> = std::result::Result<T, ConfigError>;

 #[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error(transparent)]
    ConfigFilePathError(#[from] std::io::Error),

    #[error("Failed to parse config file")]
    ConfigLoadError(String),
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
            self.blank_lines_between_sections
        )
    }
}

#[derive(Serialize, Deserialize, Debug, Builder)]
#[serde(rename_all = "PascalCase")]
pub struct TemporaryAwsCredentials {
    pub version: i32,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: String,
    pub expiration: DateTime<Utc>,
    #[serde(skip)]
    #[serde(default = "default_region")]
    pub region: String,
}

impl TemporaryAwsCredentials {
    pub fn as_json(&self) {
        let json_string = serde_json::to_string_pretty(&self).unwrap();
        println!("{json_string}");
        info!(
            "Credentials expire at {}",
            self.expiration.with_timezone(&Local)
        );
    }

    pub fn to_credentials_file(&self, config_file: &str, credentials_file: &str, profile: &str) -> Result<()> {
        let mut aws_config = Ini::new_cs();
        let mut aws_credentials = Ini::new_cs();
        aws_config.set_multiline(true);
        aws_config.set_default_section("no-section");
        aws_credentials.set_multiline(true);
        aws_credentials.set_default_section("no-section");
        let aws_credentials_file_path = expanduser(&credentials_file)?;
        let aws_config_file_path = expanduser(&config_file)?;
        aws_credentials.load(&aws_credentials_file_path).map_err(|e| ConfigError::ConfigLoadError(e))?;
        aws_config.load(&aws_config_file_path).map_err(|e| ConfigError::ConfigLoadError(e))?;

        aws_config.set(&profile, "region", Some(self.region.clone()));

        aws_credentials.set(
            &profile,
            "aws_access_key_id",
            Some(self.access_key_id.clone()),
        );
        aws_credentials.set(
            &profile,
            "aws_secret_access_key",
            Some(self.secret_access_key.clone()),
        );
        aws_credentials.set(
            &profile,
            "aws_session_token",
            Some(self.session_token.clone()),
        );

        let write_options = WriteOptionsBuilder::new()
            .space_around_delimiters(true)
            .multiline_line_indentation(2)
            .blank_lines_between_sections(1)
            .build();

        let config_res = aws_config.pretty_write(aws_config_file_path, &write_options); // TODO: handle error
        println!("{:?}", config_res);

        let creds_res = aws_credentials.pretty_write(aws_credentials_file_path, &write_options); // TODO: handle error
        println!("{:?}", creds_res);

        Ok(())
    }

    pub fn as_env_vars(&self) {
        println!(
            r#"export AWS_ACCESS_KEY_ID={}
export AWS_SECRET_ACCESS_KEY={}
export AWS_SESSION_TOKEN={}
export AWS_REGION={}
export AWS_DEFAULT_REGION={DEFAULT_REGION}
"#,
            self.access_key_id, self.secret_access_key, self.session_token, self.region
        );
    }
}

fn default_region() -> String {
    DEFAULT_REGION.to_string()
}
