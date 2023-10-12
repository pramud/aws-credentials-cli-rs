use azure_core::auth::TokenCredential;
use azure_identity::DefaultAzureCredential;

const AZ_CLIENT_ID: &str = "api://3cd4d944-d89b-401b-b2ae-fb1ece182362";
const TOKEN_EXCHANGE_URL: &str = "https://ws-iam-commontools-oidc2saml.azurewebsites.net/api/TokenExchange/SAMLResponse";
const IDENTIFIER_URI_BASE: &str = "https://signin.aws.amazon.com/saml/";

pub type Result<T> = std::result::Result<T, AzureAdTokenError>;

#[derive(Debug, thiserror::Error)]
pub enum AzureAdTokenError {
    #[error(transparent)]
    OidcTokenError(#[from] azure_core::Error),
    #[error("Azure login failed")]
    AzureLoginFailed,
    #[error(transparent)]
    SamlTokenError(#[from] reqwest::Error),
}

pub async fn oidc_token() -> Result<String> {
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
    match oidc_token().await {
        Ok(oidc_token) => saml_token_from_oidc_token(account_id, oidc_token.as_str()).await,
        Err(_error) => {
            let successful_login = std::process::Command::new("az")
                .arg("login")
                .status()
                .is_ok_and(|s| s.success());
            if successful_login {
                let oidc_token = oidc_token().await?;
                saml_token_from_oidc_token(account_id, &oidc_token).await
            } else {
                log::error!("Azure login failed");
                return Err(AzureAdTokenError::AzureLoginFailed);
            }

        }
    }
}
