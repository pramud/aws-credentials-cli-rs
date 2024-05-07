// use std::sync::Arc;

use azure_core::auth::TokenCredential;
use azure_identity::DefaultAzureCredential;

const AZ_CLIENT_ID: &str = "api://3cd4d944-d89b-401b-b2ae-fb1ece182362";
const TOKEN_EXCHANGE_URL: &str = "https://ws-iam-commontools-oidc2saml.azurewebsites.net/api/TokenExchange/SAMLResponse";
const IDENTIFIER_URI_BASE: &str = "https://signin.aws.amazon.com/saml/";

pub type Result<T> = std::result::Result<T, AzureAdTokenError>;

#[derive(Debug, thiserror::Error)]
pub enum AzureAdTokenError {
    #[error(transparent)]
    AcquireOidcTokenFailed(#[from] azure_core::Error),
    #[error(transparent)]
    AcquireSamlTokenFailed(#[from] reqwest::Error),
    #[error("Azure Login Process Error: {0}")]
    AzureLoginProcessError(String),
}

pub async fn oidc_token() -> Result<String> {
    // let az_cli_credential_arc = Arc::new(DefaultAzureCredential::default());
    // let token_credential = azure_identity::AutoRefreshingTokenCredential::new(az_cli_credential_arc);
    // log::debug!("Getting Token");
    // let response = token_credential.get_token(AZ_CLIENT_ID).await?;
    // log::debug!("Done Getting Token");
    // let oidc_token = response.token.secret();
    // Ok(oidc_token.to_string())
    let az_cli_credential = DefaultAzureCredential::default();
    let res = az_cli_credential.get_token(AZ_CLIENT_ID).await?;
    let token = res.token.secret();
    Ok(token.to_string())
}

pub async fn saml_token_from_oidc_token(account_id: &str, oidc_token: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let mut identifier_uri = String::from(IDENTIFIER_URI_BASE);
    identifier_uri.push_str(account_id);
    let saml_token = client
        .get(TOKEN_EXCHANGE_URL)
        .query(&[("IdentifierUri", identifier_uri)])
        .bearer_auth(oidc_token)
        .send()
        .await?
        .text()
        .await?;
    Ok(saml_token)
}

pub async fn saml_token(account_id: &str) -> Result<String> {
    let res = oidc_token().await;
    match res {
        Ok(oidc_token) => saml_token_from_oidc_token(account_id, &oidc_token).await,
        Err(_error) => {
            // Thank you Klaus Legarth for the Windows support
            let mut az_command = if std::env::consts::OS == "windows" {
                let mut command = std::process::Command::new("cmd");
                command.arg("/C");
                command.arg("az");
                command
            } else {
                std::process::Command::new("az")
            };
            az_command
                .arg("login")
                .status()
                .map_err(|err| AzureAdTokenError::AzureLoginProcessError(format!("{err}")))?
                .success()
                .then_some(oidc_token().await?)
                .map(|t| async move {
                    saml_token_from_oidc_token(account_id, &t).await
                })
                .unwrap()
            .await
        }
    }
}
