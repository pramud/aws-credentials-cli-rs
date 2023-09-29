use chrono::{DateTime, Utc, Local};
use derive_builder::Builder;
use log::info;
use serde::{Serialize, Deserialize};

use crate::DEFAULT_REGION;

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
        info!("Credentials expire at {}", self.expiration.with_timezone(&Local));
    }

    pub fn as_env_vars(&self) {
        println!(r#"export AWS_ACCESS_KEY_ID={}
export AWS_SECRET_ACCESS_KEY={}
export AWS_SESSION_TOKEN={}
export AWS_REGION={}
export AWS_DEFAULT_REGION={DEFAULT_REGION}
"#, self.access_key_id, self.secret_access_key, self.session_token, self.region);
    }
}

fn default_region() -> String {
    DEFAULT_REGION.to_string()
}
