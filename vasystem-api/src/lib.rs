use std::env;
use std::sync::Arc;

use http::Uri;
use tonic::transport::Channel;
pub use tonic::Request;
use tower::ServiceBuilder;

use crate::api::airlines_service_client::AirlinesServiceClient;
use crate::api::routes_service_client::RoutesServiceClient;
use crate::api::virtual_airlines_service_client::VirtualAirlinesServiceClient;
use crate::auth::AuthSvc;
use crate::oauth2::TokenSource;

pub mod api {
    #[doc(inline)]
    pub use vasystem_api_grpc::*;
}
mod auth;
mod oauth2;

pub struct Client {
    channel: AuthSvc,
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

        let mut uri: Uri = format!("https://api.{}", domain).parse()?;

        if cfg!(debug_assertions) {
            match env::var_os("VASYSTEM_API_URL") {
                Some(val) => uri = val.into_string().unwrap().parse()?,
                None => (),
            }
        }

        let channel = Channel::builder(uri).connect().await?;

        let token_source = Arc::new(token_source);

        let channel = ServiceBuilder::new()
            .layer_fn(|s| AuthSvc::new(s, token_source.clone()))
            .service(channel);

        Ok(Client { channel })
    }

    pub fn airlines(&self) -> AirlinesServiceClient<AuthSvc> {
        AirlinesServiceClient::new(self.channel.clone())
    }

    pub fn routes(&self) -> RoutesServiceClient<AuthSvc> {
        RoutesServiceClient::new(self.channel.clone())
    }

    pub fn virtual_airlines(&self) -> VirtualAirlinesServiceClient<AuthSvc> {
        VirtualAirlinesServiceClient::new(self.channel.clone())
    }
}
