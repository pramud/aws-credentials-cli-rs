use aws_sdk_sts::config::Region;

use crate::{DEFAULT_CREDS_VERSION, RoleInfo};
use super::models::{TemporaryAwsCredentials, TemporaryAwsCredentialsBuilder, TemporaryAwsCredentialsBuilderError};

use aws_sdk_sts::operation::assume_role_with_saml::AssumeRoleWithSAMLError;
use aws_smithy_types_convert::date_time::DateTimeExt;
use aws_smithy_types_convert::date_time::Error as AWSDateTimeError;

pub type Result<T> = std::result::Result<T, AwsCredentialsError>;

#[derive(Debug, thiserror::Error)]
pub enum AwsCredentialsError {
    #[error(transparent)]
    AssumeRoleFailed(#[from] aws_sdk_sts::error::SdkError<AssumeRoleWithSAMLError>),

    #[error(transparent)]
    BuildingCredentialsFailed(#[from] TemporaryAwsCredentialsBuilderError),

    #[error(transparent)]
    DateTimeError(#[from] AWSDateTimeError),
}

pub async fn acquire_aws_credentials(role_info: &RoleInfo, saml_token: &str) -> Result<TemporaryAwsCredentials> {
    let config = aws_config::defaults(aws_config::BehaviorVersion::v2023_11_09())
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
    let expiration_time = aws_creds.expiration.to_chrono_utc()?;
    let creds = TemporaryAwsCredentialsBuilder::default()
        .version(DEFAULT_CREDS_VERSION)
        .access_key_id(aws_creds.access_key_id)
        .secret_access_key(aws_creds.secret_access_key)
        .session_token(aws_creds.session_token)
        .expiration(expiration_time)
        .region(role_info.region.clone())
        .build()?;
    Ok(creds)
}
