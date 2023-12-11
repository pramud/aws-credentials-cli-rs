pub mod models;
mod azure;
mod aws;

use anyhow::Result;

use aws::acquire_aws_credentials;
use models::TemporaryAwsCredentials;

use crate::RoleInfo;

pub async fn acquire_credentials(role_info: &RoleInfo) -> Result<TemporaryAwsCredentials> {
    let saml_token = azure::saml_token(&role_info.account_id).await?;
    let credentials = acquire_aws_credentials(role_info, &saml_token).await?;
    Ok(credentials)
}

