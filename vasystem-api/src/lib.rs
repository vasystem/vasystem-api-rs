use std::sync::Arc;

use http::Uri;
use tonic::transport::Channel;
pub use tonic::Request;
use tower::ServiceBuilder;

use crate::api::airlines_service_client::AirlinesServiceClient;
use crate::api::routes_service_client::RoutesServiceClient;
use crate::auth::AuthSvc;
use crate::oauth2::TokenSource;

pub mod api {
    #[doc(inline)]
    pub use vasystem_api_grpc::*;
}
mod auth;
mod oauth2;

pub struct Client {
    channel: AuthSvc<Channel>,
}

impl Client {
    pub async fn connect(
        domain: String,
        client_id: String,
        client_secret: String,
        scopes: Vec<String>,
    ) -> Result<Client, Box<dyn std::error::Error>> {
        let token_source =
            TokenSource::new(domain.clone(), client_id, client_secret, scopes).await?;

        let uri: Uri = format!("https://api.{}", domain).parse()?;
        let channel = Channel::builder(uri).connect().await?;

        let token_source = Arc::new(token_source);

        let channel = ServiceBuilder::new()
            .layer_fn(|s| AuthSvc::new(s, token_source.clone()))
            .service(channel);

        Ok(Client { channel })
    }

    pub fn airlines(&self) -> AirlinesServiceClient<AuthSvc<Channel>> {
        AirlinesServiceClient::new(self.channel.clone())
    }

    pub fn routes(&self) -> RoutesServiceClient<AuthSvc<Channel>> {
        RoutesServiceClient::new(self.channel.clone())
    }
}
