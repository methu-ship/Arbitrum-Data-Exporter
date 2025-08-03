use ethers::providers::{Provider, Http};
use std::env;
use std::net::SocketAddr;
use tokio;

mod metrics;
mod updater;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let rpc_url = env::var("RPC_URL")?;
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let metrics = metrics::ArbitrumMetrics::new()?;

    let addr: SocketAddr = ([0, 0, 0, 0], 8000).into();
    let updater_handle = tokio::spawn(updater::run_updater(
        provider.clone(),
        metrics.gas_price_gauge.clone(),
        metrics.latest_block_gauge.clone(),
    ));

    let server_handle = tokio::spawn(server::run_server(metrics.registry, addr));


    tokio::try_join!(updater_handle, server_handle)?;

    Ok(())
}
