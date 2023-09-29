use std::error::Error;

use aws::acquire_aws_credentials;
use azure::{oidc_token, saml_token};
use models::TemporaryAwsCredentials;

use crate::RoleInfo;

mod azure;
pub mod aws;
pub mod models;

pub async fn acquire_credentials(role_info: &RoleInfo) -> Result<TemporaryAwsCredentials, Box<dyn Error>> {
    let oidc_token = oidc_token().await?;
    let saml_token = saml_token(&role_info.account_id, &oidc_token).await?;
    acquire_aws_credentials(role_info, &saml_token).await
}

