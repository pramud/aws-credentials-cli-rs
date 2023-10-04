use std::error::Error;
use std::time::SystemTime;

use aws_sdk_sts::config::Region;

use crate::{DEFAULT_CREDS_VERSION, RoleInfo};
use super::models::{TemporaryAwsCredentials, TemporaryAwsCredentialsBuilder};


pub async fn acquire_aws_credentials(role_info: &RoleInfo, saml_token: &str) -> Result<TemporaryAwsCredentials, Box<dyn Error>> {
    let config = aws_config::from_env()
        .no_credentials()
        .region(Region::new(role_info.region.clone()))
        .load()
        .await;

    let principal_arn = format!("arn:aws:iam::{}:saml-provider/AzureAD", role_info.account_id);
    let sts_client = aws_sdk_sts::Client::new(&config);
    let result = sts_client.assume_role_with_saml()
        .role_arn(role_info.role_arn())
        .principal_arn(principal_arn)
        .saml_assertion(saml_token)
        .duration_seconds(role_info.duration)
        .send()
        .await?;
    let aws_creds = result.credentials.unwrap();
    let expiration_as_system_time = SystemTime::try_from(aws_creds.expiration.unwrap())?;
    let creds = TemporaryAwsCredentialsBuilder::default()
        .version(DEFAULT_CREDS_VERSION)
        .access_key_id(aws_creds.access_key_id.unwrap())
        .secret_access_key(aws_creds.secret_access_key.unwrap())
        .session_token(aws_creds.session_token.unwrap())
        .expiration(expiration_as_system_time.into())
        .region(role_info.region.clone())
        .build()?;
    Ok(creds)
}
