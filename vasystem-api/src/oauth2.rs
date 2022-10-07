use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::reqwest::async_http_client;
use oauth2::{AuthUrl, ClientId, ClientSecret, Scope, TokenResponse, TokenUrl};
use reqwest;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct WellKnown {
    pub authorization_endpoint: String,
    pub token_endpoint: String,
}

struct State {
    token: Option<BasicTokenResponse>,
    expiry: Option<std::time::SystemTime>,
}

pub struct TokenSource {
    client: Box<BasicClient>,
    pub scopes: Vec<Scope>,
    mutex: Arc<tokio::sync::Mutex<State>>,
}

pub async fn get_well_known(domain: &str) -> Result<WellKnown, Box<dyn std::error::Error>> {
    let well_known: WellKnown = reqwest::get(format!(
        "https://{}/.well-known/openid-configuration",
        domain
    ))
    .await?
    .json()
    .await?;

    Ok(well_known)
}

pub fn create_client(
    well_known: WellKnown,
    client_id: String,
    client_secret: String,
) -> Result<BasicClient, Box<dyn std::error::Error>> {
    let client = BasicClient::new(
        ClientId::new(client_id),
        Some(ClientSecret::new(client_secret)),
        AuthUrl::new(well_known.authorization_endpoint)?,
        Some(TokenUrl::new(well_known.token_endpoint)?),
    );

    Ok(client)
}

impl TokenSource {
    pub async fn new(
        domain: String,
        client_id: String,
        client_secret: String,
        scopes: Vec<String>,
    ) -> Result<TokenSource, Box<dyn std::error::Error>> {
        let well_known = get_well_known(domain.as_str()).await?;
        let client = create_client(well_known, client_id, client_secret)?;

        let token_source = TokenSource {
            client: Box::new(client),
            scopes: scopes.iter().map(|s| Scope::new(s.to_string())).collect(),
            mutex: Arc::new(tokio::sync::Mutex::new(State {
                token: None,
                expiry: None,
            })),
        };

        Ok(token_source)
    }

    pub async fn access_token(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut state = self.mutex.lock().await;

        if state.expiry.is_none() || state.expiry.unwrap() < std::time::SystemTime::now() {
            let token_result = self
                .client
                .exchange_client_credentials()
                .add_scopes(self.scopes.clone())
                .request_async(async_http_client)
                .await?;

            state.token = Some(token_result);

            let duration = match state.token.as_ref().unwrap().expires_in() {
                Some(duration) => duration,
                None => std::time::Duration::from_secs(3600),
            };

            state.expiry = Some(std::time::SystemTime::now() + duration);
        }

        Ok(state
            .token
            .as_ref()
            .unwrap()
            .access_token()
            .secret()
            .to_string())
    }
}
