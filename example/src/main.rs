use clap::Parser;

use vasystem_api::{
    api::{GetVirtualAirlineRequest, ListAirlinesRequest, ListRoutesRequest},
    Client, Request,
};

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    domain: String,
    #[arg(long)]
    client_id: String,
    #[arg(long)]
    client_secret: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let client = Client::connect(
        args.domain,
        args.client_id,
        args.client_secret,
        vec!["airlines".to_string(), "routes".to_string()],
    )
    .await?;

    let response = client
        .virtual_airlines()
        .get_virtual_airline(Request::new(GetVirtualAirlineRequest {}))
        .await?;

    println!("RESPONSE = {:?}", response);

    let response = client
        .airlines()
        .list_airlines(Request::new(ListAirlinesRequest {}))
        .await?;

    println!("RESPONSE = {:?}", response);

    let response = client
        .routes()
        .list_routes(Request::new(ListRoutesRequest {
            ..Default::default()
        }))
        .await?;

    println!("RESPONSE = {:?}", response);

    Ok(())
}
