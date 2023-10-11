pub mod models;
mod azure;
mod aws;

use std::error::Error;

use aws::acquire_aws_credentials;
use azure::saml_token;
use models::TemporaryAwsCredentials;

use crate::RoleInfo;

pub async fn acquire_credentials(role_info: &RoleInfo) -> Result<TemporaryAwsCredentials, Box<dyn Error>> {
    let saml_token = saml_token(&role_info.account_id).await?;
    acquire_aws_credentials(role_info, &saml_token).await
}

