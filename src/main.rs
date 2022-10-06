pub mod vasystemapi {
    tonic::include_proto!("vasystem.api.v2");
}

use clap::Parser;
use reqwest;
use oauth2::{AuthUrl, ClientId, ClientSecret, Scope, TokenResponse, TokenUrl};
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use serde::{Deserialize};
use tonic::{metadata::MetadataValue, Request, transport::Channel};
use tonic::transport::Uri;

use vasystemapi::airlines_service_client::AirlinesServiceClient;
use vasystemapi::{ListAirlinesRequest};

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    domain: String,
    #[arg(long)]
    client_id: String,
    #[arg(long)]
    client_secret: String,
}

#[derive(Deserialize)]
struct WellKnown {
    authorization_endpoint: String,
    token_endpoint: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let body: WellKnown = reqwest::get(format!("https://{}/.well-known/openid-configuration", args.domain))
        .await?
        .json()
        .await?;

    let oauth2_client = BasicClient::new(
        ClientId::new(args.client_id),
        Some(ClientSecret::new(args.client_secret)),
        AuthUrl::new(body.authorization_endpoint)?,
        Some(TokenUrl::new(body.token_endpoint)?),
    );

    let token_result = oauth2_client
        .exchange_client_credentials()
        .add_scope(Scope::new("airlines".to_string()))
        .add_scope(Scope::new("routes".to_string()))
        .request_async(async_http_client).await?;

    let uri: Uri = format!("https://api.{}:443", args.domain).parse()?;
    let channel = Channel::builder(uri).connect().await?;

    let token: MetadataValue<_> = format!("Bearer {}", token_result.access_token().secret()).parse()?;

    let mut client = AirlinesServiceClient::with_interceptor(channel, move |mut req: Request<()>| {
        req.metadata_mut().insert("authorization", token.clone());
        Ok(req)
    });

    let response = client.list_airlines(Request::new(ListAirlinesRequest {})).await?;

    println!("RESPONSE = {:?}", response);

    Ok(())
}
