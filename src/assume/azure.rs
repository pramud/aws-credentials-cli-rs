use std::error::Error;

use azure_core::auth::TokenCredential;
use azure_identity::DefaultAzureCredential;

const AZ_CLIENT_ID: &str = "api://3cd4d944-d89b-401b-b2ae-fb1ece182362";
const TOKEN_EXCHANGE_URL: &str = "https://ws-iam-commontools-oidc2saml.azurewebsites.net/api/TokenExchange/SAMLResponse";
const IDENTIFIER_URI_BASE: &str = "https://signin.aws.amazon.com/saml/";

pub async fn oidc_token() -> Result<String, Box<dyn Error>> {
    let az_cli_credential = DefaultAzureCredential::default();
    let res = az_cli_credential.get_token(AZ_CLIENT_ID).await?;
    let token = res.token.secret();
    Ok(token.to_string())
}

pub async fn saml_token_from_oidc_token(account_id: &str, oidc_token: &String) -> Result<String, Box<dyn Error>> {
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

pub async fn saml_token(account_id: &str) -> Result<String, Box<dyn Error>> {
    let oidc_token = oidc_token().await?;
    saml_token_from_oidc_token(account_id, &oidc_token).await
}
